use crate::error::UpnpError;
use easy_upnp::PortMappingProtocol;
use log::{debug, error, info, trace};
use obsidian_scheduler::callback::CallbackTimer;
use obsidian_scheduler::timer_trait::Timer;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

/// Renewal interval: how often port leases are refreshed.
const RENEWAL_INTERVAL_SECS: u64 = 300; // 5 minutes

/// Lease duration passed to the router. Set to 2x the renewal interval
/// to provide a safety buffer if a renewal cycle is delayed.
const LEASE_DURATION_SECS: u32 = 600; // 10 minutes

/// Represents a single active UPnP port mapping.
#[derive(Clone, Debug)]
pub struct PortMapping {
    pub port: u16,
    pub description: String,
    pub protocol: PortMappingProtocol,
}

/// Internal state holding all active port mappings and the renewal timer.
struct UpnpState {
    ports: Vec<PortMapping>,
    renewal_timer: Option<Arc<CallbackTimer>>,
}

/// Thread-safe, async-first UPnP port manager.
///
/// Provides methods to add and remove port mappings on the local router
/// via UPnP IGD. Port leases are automatically renewed every 5 minutes
/// using a [`CallbackTimer`] from `obsidian-scheduler`.
///
/// Access the global singleton via [`UpnpManager::global()`].
pub struct UpnpManager {
    state: Mutex<UpnpState>,
}

static INSTANCE: OnceLock<UpnpManager> = OnceLock::new();

impl UpnpManager {
    /// Get the global `UpnpManager` instance, creating it on first call.
    pub fn global() -> &'static UpnpManager {
        INSTANCE.get_or_init(|| UpnpManager {
            state: Mutex::new(UpnpState {
                ports: Vec::new(),
                renewal_timer: None,
            }),
        })
    }

    /// Register a port mapping with the router.
    ///
    /// The port is immediately forwarded via UPnP. If this is the first
    /// active port, the automatic renewal timer is started.
    ///
    /// Returns [`UpnpError::PortAlreadyMapped`] if the port is already tracked.
    pub async fn add_port(
        &self,
        port: u16,
        description: String,
        protocol: PortMappingProtocol,
    ) -> Result<(), UpnpError> {
        // Phase 1: check for duplicate (short lock)
        {
            let state = self.state.lock().await;
            if state.ports.iter().any(|p| p.port == port) {
                return Err(UpnpError::PortAlreadyMapped(port));
            }
        }

        // Phase 2: blocking UPnP call (no lock held)
        let desc_clone = description.clone();
        tokio::task::spawn_blocking(move || {
            let config = easy_upnp::UpnpConfig {
                address: None,
                port,
                protocol,
                duration: LEASE_DURATION_SECS,
                comment: desc_clone,
            };
            for result in easy_upnp::add_ports(vec![config]) {
                if let Err(e) = result {
                    return Err(UpnpError::UpnpOperationFailed(format!(
                        "failed to forward port {port}: {e}"
                    )));
                }
            }
            Ok(())
        })
        .await
        .map_err(|e| UpnpError::UpnpOperationFailed(e.to_string()))??;

        // Phase 3: store mapping and ensure renewal timer (short lock)
        {
            let mut state = self.state.lock().await;
            // Re-check in case of concurrent add
            if !state.ports.iter().any(|p| p.port == port) {
                state.ports.push(PortMapping {
                    port,
                    description,
                    protocol,
                });
            }
            if state.renewal_timer.is_none() {
                self.start_renewal_timer(&mut state).await?;
            }
        }

        info!("UPnP port {port} mapped successfully");
        Ok(())
    }

    /// Remove a port mapping from the router.
    ///
    /// The port is deleted from the router on a best-effort basis (failures
    /// are logged but not propagated). If this was the last active port,
    /// the renewal timer is stopped.
    ///
    /// Returns [`UpnpError::PortNotFound`] if the port is not tracked.
    pub async fn remove_port(&self, port: u16) -> Result<(), UpnpError> {
        // Phase 1: find and remove from tracking (short lock)
        let mapping = {
            let mut state = self.state.lock().await;
            let index = state
                .ports
                .iter()
                .position(|p| p.port == port)
                .ok_or(UpnpError::PortNotFound(port))?;
            let mapping = state.ports.remove(index);

            // Stop renewal if no ports remain
            if state.ports.is_empty() {
                Self::stop_renewal_timer(&mut state).await;
            }
            mapping
        };

        // Phase 2: best-effort UPnP delete (no lock held)
        tokio::task::spawn_blocking(move || {
            let config = easy_upnp::UpnpConfig {
                address: None,
                port: mapping.port,
                protocol: mapping.protocol,
                duration: 0,
                comment: String::new(),
            };
            for result in easy_upnp::delete_ports(vec![config]) {
                if let Err(e) = result {
                    error!("Failed to delete UPnP port {}: {}", mapping.port, e);
                }
            }
        })
        .await
        .map_err(|e| UpnpError::UpnpOperationFailed(e.to_string()))?;

        info!("UPnP port {port} removed");
        Ok(())
    }

    /// Remove all port mappings. Intended for shutdown cleanup.
    ///
    /// Stops the renewal timer and deletes all tracked ports from the
    /// router on a best-effort basis.
    pub async fn remove_all_ports(&self) -> Result<(), UpnpError> {
        let mappings = {
            let mut state = self.state.lock().await;
            Self::stop_renewal_timer(&mut state).await;
            std::mem::take(&mut state.ports)
        };

        if mappings.is_empty() {
            return Ok(());
        }

        info!("Removing all {} UPnP port mappings", mappings.len());

        tokio::task::spawn_blocking(move || {
            let configs: Vec<easy_upnp::UpnpConfig> = mappings
                .iter()
                .map(|m| easy_upnp::UpnpConfig {
                    address: None,
                    port: m.port,
                    protocol: m.protocol,
                    duration: 0,
                    comment: String::new(),
                })
                .collect();

            for result in easy_upnp::delete_ports(configs) {
                if let Err(e) = result {
                    error!("Failed to delete UPnP port: {e}");
                }
            }
        })
        .await
        .map_err(|e| UpnpError::UpnpOperationFailed(e.to_string()))?;

        info!("All UPnP ports removed");
        Ok(())
    }

    /// Return a snapshot of all currently active port mappings.
    pub async fn get_ports(&self) -> Vec<PortMapping> {
        let state = self.state.lock().await;
        state.ports.clone()
    }

    /// Check whether a specific port is currently mapped.
    pub async fn has_port(&self, port: u16) -> bool {
        let state = self.state.lock().await;
        state.ports.iter().any(|p| p.port == port)
    }

    /// Start the renewal timer. Called internally when the first port is added.
    async fn start_renewal_timer(&self, state: &mut UpnpState) -> Result<(), UpnpError> {
        let timer = CallbackTimer::new(
            move |_handle| async move {
                let manager = UpnpManager::global();
                let ports = {
                    let state = manager.state.lock().await;
                    state.ports.clone()
                };

                if ports.is_empty() {
                    return Ok(());
                }

                debug!("Renewing {} UPnP port mappings", ports.len());

                let configs: Vec<easy_upnp::UpnpConfig> = ports
                    .iter()
                    .map(|p| easy_upnp::UpnpConfig {
                        address: None,
                        port: p.port,
                        protocol: p.protocol,
                        duration: LEASE_DURATION_SECS,
                        comment: p.description.clone(),
                    })
                    .collect();

                tokio::task::spawn_blocking(move || {
                    for result in easy_upnp::add_ports(configs) {
                        match result {
                            Ok(()) => trace!("Port renewed"),
                            Err(e) => error!("Port renewal failed: {e}"),
                        }
                    }
                })
                .await
                .map_err(|e| anyhow::anyhow!("renewal spawn_blocking failed: {e}"))?;

                Ok(())
            },
            std::time::Duration::from_secs(RENEWAL_INTERVAL_SECS),
        );

        timer
            .start()
            .await
            .map_err(|e| UpnpError::RenewalError(e.to_string()))?;

        state.renewal_timer = Some(timer);
        debug!(
            "UPnP renewal timer started (interval: {RENEWAL_INTERVAL_SECS}s)"
        );
        Ok(())
    }

    /// Stop the renewal timer if running.
    async fn stop_renewal_timer(state: &mut UpnpState) {
        if let Some(timer) = state.renewal_timer.take() {
            let _ = timer.stop().await;
            debug!("UPnP renewal timer stopped");
        }
    }
}
