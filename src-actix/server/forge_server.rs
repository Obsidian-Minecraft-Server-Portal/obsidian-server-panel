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
    /// needs to be run before the server can start. Installer jars are also
    /// recognized by name alone, because modpack server packs frequently ship
    /// without manifest metadata tying them to a loader type.
    pub(crate) fn is_forge_installer(&self) -> bool {
        let jar = self.server_jar.to_lowercase();
        let looks_like_installer = jar.contains("installer") && (jar.contains("forge") || jar.contains("neoforge"));
        match &self.server_type {
            Some(ServerType::Forge | ServerType::NeoForge) => {
                looks_like_installer || (jar.contains("forge") && !jar.contains("server"))
            }
            _ => looks_like_installer,
        }
    }

    /// Fills in the server type and loader/minecraft versions from an installer jar
    /// name like `forge-1.20.1-47.3.0-installer.jar` or `neoforge-20.4.237-installer.jar`
    /// when the server record lacks them, so modpack servers without manifest
    /// metadata can still run their installer.
    fn derive_loader_info_from_installer(&mut self) {
        let jar = self.server_jar.trim_end_matches(".jar").to_lowercase();
        let parts: Vec<&str> = jar.split('-').collect();
        match parts.as_slice() {
            ["forge", minecraft_version, forge_version, ..] => {
                if !matches!(self.server_type, Some(ServerType::Forge)) {
                    self.server_type = Some(ServerType::Forge);
                }
                if self.minecraft_version.is_none() {
                    self.minecraft_version = Some(minecraft_version.to_string());
                }
                if self.loader_version.is_none() {
                    self.loader_version = Some(forge_version.to_string());
                }
            }
            ["neoforge", neoforge_version, ..] => {
                if !matches!(self.server_type, Some(ServerType::NeoForge)) {
                    self.server_type = Some(ServerType::NeoForge);
                }
                if self.loader_version.is_none() {
                    self.loader_version = Some(neoforge_version.to_string());
                }
            }
            _ => {}
        }
    }

    /// Whether the server directory has neither a runnable server jar nor an installer jar,
    /// so starting it must first download and run the loader installer.
    pub(crate) fn needs_loader_install(&self) -> bool {
        matches!(self.server_type, Some(ServerType::Forge | ServerType::NeoForge)) && !self.is_forge_installer() && !self.has_launchable_jar()
    }

    /// Runs the Forge or NeoForge installer, delegating to the appropriate
    /// loader crate. Tracks the installation process in `ACTIVE_SERVERS`
    /// for status monitoring.
    pub(crate) async fn install_forge_server(&mut self) -> Result<()> {
        self.status = ServerStatus::Starting;
        self.save().await?;

        if let Err(e) = self.install_forge_server_inner().await {
            self.settle_failed_start().await;
            return Err(e);
        }
        Ok(())
    }

    async fn install_forge_server_inner(&mut self) -> Result<()> {
        let directory_path = self.get_directory_path().canonicalize()?;
        let server_id = self.id;
        let mut self_clone = self.clone();

        self_clone.derive_loader_info_from_installer();
        self_clone.save().await.ok();

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
        servers.lock().await.insert(self.id, crate::server::server_actions::ServerHandle::Direct(0));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::server_type::ServerType;

    fn server(jar: &str, server_type: ServerType) -> ServerData {
        crate::settings::initialize_settings_path();
        let mut server = ServerData::new("test".into(), server_type, "1.20.1".into(), None, "java".into(), 1);
        server.server_jar = jar.to_string();
        server
    }

    #[test]
    fn detects_installer_jars_without_a_loader_type() {
        assert!(server("forge-1.16.5-36.2.39-installer.jar", ServerType::Custom).is_forge_installer());
        assert!(server("neoforge-20.4.237-installer.jar", ServerType::Custom).is_forge_installer());
        assert!(!server("minecraft_server.jar", ServerType::Custom).is_forge_installer());
    }

    #[test]
    fn derives_loader_info_from_installer_names() {
        let mut forge = server("forge-1.20.1-47.3.0-installer.jar", ServerType::Custom);
        forge.derive_loader_info_from_installer();
        assert_eq!(forge.server_type, Some(ServerType::Forge));
        assert_eq!(forge.minecraft_version.as_deref(), Some("1.20.1"));
        assert_eq!(forge.loader_version.as_deref(), Some("47.3.0"));

        let mut neoforge = server("neoforge-20.4.237-installer.jar", ServerType::Custom);
        neoforge.derive_loader_info_from_installer();
        assert_eq!(neoforge.server_type, Some(ServerType::NeoForge));
        assert_eq!(neoforge.loader_version.as_deref(), Some("20.4.237"));
    }
}
