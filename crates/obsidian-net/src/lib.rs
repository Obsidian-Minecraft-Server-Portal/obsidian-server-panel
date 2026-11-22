use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use igd_next::aio::Gateway;
use igd_next::aio::tokio::{Tokio, search_gateway};
use igd_next::{PortMappingProtocol, SearchOptions};
use obsidian_common::{CommonError, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;
use utoipa::ToSchema;

const LEASE_SECS: u32 = 7200;
const RENEW_EVERY: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
	Tcp,
	Udp,
}

impl From<Protocol> for PortMappingProtocol {
	fn from(protocol: Protocol) -> Self {
		match protocol {
			Protocol::Tcp => Self::TCP,
			Protocol::Udp => Self::UDP,
		}
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UpnpStatus {
	pub mapped: bool,
	#[schema(value_type = Option<String>)]
	pub external_ip: Option<IpAddr>,
	pub external_port: Option<u16>,
}

struct Lease {
	protocol: Protocol,
	renewal: JoinHandle<()>,
}

#[derive(Default)]
pub struct UpnpManager {
	gateway: AsyncMutex<Option<Arc<Gateway<Tokio>>>>,
	leases: Mutex<HashMap<u16, Lease>>,
}

fn other(error: impl std::fmt::Display) -> CommonError {
	CommonError::Other(format!("upnp: {error}"))
}

impl UpnpManager {
	pub fn new() -> Self {
		Self::default()
	}

	pub async fn map(&self, port: u16, protocol: Protocol, description: &str) -> Result<()> {
		let gateway = self.gateway().await?;
		let local = SocketAddr::new(local_ip_address::local_ip().map_err(other)?, port);
		gateway
			.add_port(protocol.into(), port, local, LEASE_SECS, description)
			.await
			.map_err(other)?;
		let renewal = tokio::spawn(renew(Arc::clone(&gateway), protocol, port, local, description.to_owned()));
		if let Some(previous) = self.leases().insert(port, Lease { protocol, renewal }) {
			previous.renewal.abort();
		}
		Ok(())
	}

	pub async fn unmap(&self, port: u16) -> Result<()> {
		let Some(lease) = self.leases().remove(&port) else {
			return Ok(());
		};
		lease.renewal.abort();
		self.gateway().await?.remove_port(lease.protocol.into(), port).await.map_err(other)
	}

	pub async fn external_ip(&self) -> Result<IpAddr> {
		self.gateway().await?.get_external_ip().await.map_err(other)
	}

	pub async fn status(&self, port: u16) -> Result<UpnpStatus> {
		let mapped = self.leases().contains_key(&port);
		let external_ip = if mapped { self.external_ip().await.ok() } else { None };
		Ok(UpnpStatus { mapped, external_ip, external_port: mapped.then_some(port) })
	}

	async fn gateway(&self) -> Result<Arc<Gateway<Tokio>>> {
		let mut slot = self.gateway.lock().await;
		if let Some(gateway) = slot.as_ref() {
			return Ok(Arc::clone(gateway));
		}
		let gateway = Arc::new(search_gateway(SearchOptions::default()).await.map_err(other)?);
		*slot = Some(Arc::clone(&gateway));
		Ok(gateway)
	}

	fn leases(&self) -> MutexGuard<'_, HashMap<u16, Lease>> {
		self.leases.lock().unwrap_or_else(PoisonError::into_inner)
	}
}

async fn renew(gateway: Arc<Gateway<Tokio>>, protocol: Protocol, port: u16, local: SocketAddr, description: String) {
	let mut ticker = tokio::time::interval(RENEW_EVERY);
	ticker.tick().await;
	loop {
		ticker.tick().await;
		if let Err(error) = gateway.add_port(protocol.into(), port, local, LEASE_SECS, &description).await {
			tracing::warn!(%error, port, "failed to renew UPnP mapping");
		}
	}
}
