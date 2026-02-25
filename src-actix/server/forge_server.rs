use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use crate::server::server_type::ServerType;
use anyhow::{anyhow, Result};
use log::{debug, error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

impl ServerData {
    /// Checks whether this server has a Forge or NeoForge installer that
    /// needs to be run before the server can start.
    pub(crate) fn is_forge_installer(&self) -> bool {
        if let Some(server_type) = &self.server_type
            && (*server_type == ServerType::Forge || *server_type == ServerType::NeoForge)
        {
            return self.server_jar.contains("installer")
                || (self.server_jar.contains("forge")
                    && !self.server_jar.contains("server"));
        }
        false
    }

    /// Runs the Forge or NeoForge installer, delegating to the appropriate
    /// loader crate. Tracks the installation process in `ACTIVE_SERVERS`
    /// for status monitoring.
    pub(crate) async fn install_forge_server(&mut self) -> Result<()> {
        self.status = ServerStatus::Starting;
        self.save().await?;

        let directory_path = self.get_directory_path().canonicalize()?;
        let server_id = self.id;
        let mut self_clone = self.clone();

        debug!("Installing loader server {}", server_id);

        // Spawn the installation in a background task so we can track it
        // and update server state on completion.
        tokio::spawn(async move {
            let result = match self_clone.server_type {
                Some(ServerType::Forge) => {
                    let mc_version = match self_clone.minecraft_version.as_ref() {
                        Some(v) => v.clone(),
                        None => {
                            error!("No Minecraft version set for Forge server {}", server_id);
                            if let Err(e) = self_clone.remove_server_crashed().await {
                                error!("Failed to update crashed server status: {}", e);
                            }
                            return;
                        }
                    };
                    let forge_version = match self_clone.loader_version.as_ref() {
                        Some(v) => v.clone(),
                        None => {
                            error!("No loader version set for Forge server {}", server_id);
                            if let Err(e) = self_clone.remove_server_crashed().await {
                                error!("Failed to update crashed server status: {}", e);
                            }
                            return;
                        }
                    };

                    let client = forge_loader::ForgeClient::new();
                    client
                        .install_server(forge_loader::ForgeInstallOptions {
                            mc_version: &mc_version,
                            forge_version: &forge_version,
                            install_dir: &directory_path,
                            java_executable: &self_clone.java_executable,
                            download_progress: None,
                        })
                        .await
                        .map(|r| (r.java_args, r.server_jar))
                        .map_err(|e| anyhow!("{}", e))
                }
                Some(ServerType::NeoForge) => {
                    let neoforge_version = match self_clone.loader_version.as_ref() {
                        Some(v) => v.clone(),
                        None => {
                            error!("No loader version set for NeoForge server {}", server_id);
                            if let Err(e) = self_clone.remove_server_crashed().await {
                                error!("Failed to update crashed server status: {}", e);
                            }
                            return;
                        }
                    };

                    let client = neoforge_loader::NeoForgeClient::new();
                    client
                        .install_server(neoforge_loader::NeoForgeInstallOptions {
                            neoforge_version: &neoforge_version,
                            install_dir: &directory_path,
                            java_executable: &self_clone.java_executable,
                            download_progress: None,
                        })
                        .await
                        .map(|r| (r.java_args, r.server_jar))
                        .map_err(|e| anyhow!("{}", e))
                }
                _ => {
                    error!("install_forge_server called on non-Forge/NeoForge server {}", server_id);
                    if let Err(e) = self_clone.remove_server_crashed().await {
                        error!("Failed to update crashed server status: {}", e);
                    }
                    return;
                }
            };

            match result {
                Ok((java_args, server_jar)) => {
                    self_clone.java_args = java_args;
                    self_clone.server_jar = server_jar;
                    if let Err(e) = self_clone.remove_server().await {
                        error!("Failed to remove server from active list: {}", e);
                    }
                }
                Err(e) => {
                    error!("Loader installation failed for server {}: {}", server_id, e);
                    if let Err(e) = self_clone.remove_server_crashed().await {
                        error!("Failed to update crashed server status: {}", e);
                    }
                }
            }
        });

        // Track the server as active during installation
        let servers = crate::server::server_actions::ACTIVE_SERVERS
            .get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        servers.lock().await.insert(self.id, 0);

        Ok(())
    }
}
