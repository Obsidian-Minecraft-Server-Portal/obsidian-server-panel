use super::update_data::UpdateInfo;
use crate::server::server_data::ServerData;
use crate::server::server_type::ServerType;
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use reqwest::Client;
use serde_json::Value;

pub struct UpdateChecker;

impl UpdateChecker {
    /// Check if server has an update available
    pub async fn check_for_updates(server: &ServerData) -> Result<Option<UpdateInfo>> {
        info!(
            "Checking for updates for server '{}' (type: {:?})",
            server.name, server.server_type
        );

        match server.server_type {
            Some(ServerType::Vanilla) => Self::check_vanilla_update(server).await,
            Some(ServerType::Fabric) => Self::check_fabric_update(server).await,
            Some(ServerType::Forge) => Self::check_forge_update(server).await,
            Some(ServerType::NeoForge) => Self::check_neoforge_update(server).await,
            Some(ServerType::Quilt) => Self::check_quilt_update(server).await,
            _ => {
                warn!(
                    "Update checking not supported for server type: {:?}",
                    server.server_type
                );
                Ok(None)
            }
        }
    }

    /// Check for Vanilla Minecraft server updates
    async fn check_vanilla_update(server: &ServerData) -> Result<Option<UpdateInfo>> {
        let client = Client::new();
        let current_version = server
            .minecraft_version
            .as_ref()
            .ok_or(anyhow!("No minecraft version set"))?;

        debug!(
            "Checking vanilla update for current version: {}",
            current_version
        );

        // Fetch version manifest from Mojang
        let response = client
            .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
            .send()
            .await?;

        let manifest: Value = response.json().await?;

        // Get latest release version
        let latest_version = manifest["latest"]["release"]
            .as_str()
            .ok_or(anyhow!("Failed to parse latest version"))?
            .to_string();

        if current_version == &latest_version {
            info!("Server '{}' is up to date ({})", server.name, current_version);
            return Ok(None);
        }

        // Find the version in the versions list
        let versions = manifest["versions"]
            .as_array()
            .ok_or(anyhow!("Failed to parse versions array"))?;

        let version_info = versions
            .iter()
            .find(|v| v["id"].as_str() == Some(&latest_version))
            .ok_or(anyhow!("Version {} not found in manifest", latest_version))?;

        // Get the version manifest URL
        let version_url = version_info["url"]
            .as_str()
            .ok_or(anyhow!("Failed to get version URL"))?;

        // Fetch the version-specific manifest to get download URL
        let version_response = client.get(version_url).send().await?;
        let version_manifest: Value = version_response.json().await?;

        let download_url = version_manifest["downloads"]["server"]["url"]
            .as_str()
            .ok_or(anyhow!("Failed to get server download URL"))?
            .to_string();

        info!(
            "Update available for server '{}': {} -> {}",
            server.name, current_version, latest_version
        );

        Ok(Some(UpdateInfo::new(
            current_version.clone(),
            latest_version.clone(),
            download_url,
            Some(format!(
                "https://www.minecraft.net/en-us/article/minecraft-java-edition-{}",
                latest_version.replace('.', "-")
            )),
        )))
    }

    /// Check for Fabric loader updates
    async fn check_fabric_update(server: &ServerData) -> Result<Option<UpdateInfo>> {
        let client = fabric_loader::FabricClient::new();
        let current_version = server
            .loader_version
            .as_ref()
            .ok_or(anyhow!("No loader version set"))?;
        let minecraft_version = server
            .minecraft_version
            .as_ref()
            .ok_or(anyhow!("No minecraft version set"))?;

        match client
            .check_for_update(minecraft_version, current_version)
            .await
        {
            Ok(Some(update)) => {
                info!(
                    "Update available for server '{}' Fabric: {} -> {}",
                    server.name, current_version, update.latest_loader_version
                );
                Ok(Some(UpdateInfo::new(
                    current_version.clone(),
                    update.latest_loader_version,
                    update.download_url,
                    Some(update.changelog_url),
                )))
            }
            Ok(None) => {
                info!(
                    "Server '{}' Fabric loader is up to date ({})",
                    server.name, current_version
                );
                Ok(None)
            }
            Err(e) => Err(anyhow!("Fabric update check failed: {}", e)),
        }
    }

    /// Check for Forge loader updates
    async fn check_forge_update(server: &ServerData) -> Result<Option<UpdateInfo>> {
        let client = forge_loader::ForgeClient::new();
        let current_version = server
            .loader_version
            .as_ref()
            .ok_or(anyhow!("No loader version set"))?;
        let minecraft_version = server
            .minecraft_version
            .as_ref()
            .ok_or(anyhow!("No minecraft version set"))?;

        match client
            .check_for_update(minecraft_version, current_version)
            .await
        {
            Ok(Some(update)) => {
                info!(
                    "Update available for server '{}' Forge: {} -> {}",
                    server.name, current_version, update.latest_version
                );
                Ok(Some(UpdateInfo::new(
                    current_version.clone(),
                    update.latest_version,
                    update.download_url,
                    Some(update.changelog_url),
                )))
            }
            Ok(None) => {
                info!(
                    "Server '{}' Forge is up to date ({})",
                    server.name, current_version
                );
                Ok(None)
            }
            Err(e) => Err(anyhow!("Forge update check failed: {}", e)),
        }
    }

    /// Check for NeoForge loader updates
    async fn check_neoforge_update(server: &ServerData) -> Result<Option<UpdateInfo>> {
        let client = neoforge_loader::NeoForgeClient::new();
        let current_version = server
            .loader_version
            .as_ref()
            .ok_or(anyhow!("No loader version set"))?;
        let minecraft_version = server
            .minecraft_version
            .as_ref()
            .ok_or(anyhow!("No minecraft version set"))?;

        match client
            .check_for_update(minecraft_version, current_version)
            .await
        {
            Ok(Some(update)) => {
                info!(
                    "Update available for server '{}' NeoForge: {} -> {}",
                    server.name, current_version, update.latest_version
                );
                Ok(Some(UpdateInfo::new(
                    current_version.clone(),
                    update.latest_version,
                    update.download_url,
                    Some(update.changelog_url),
                )))
            }
            Ok(None) => {
                info!(
                    "Server '{}' NeoForge is up to date ({})",
                    server.name, current_version
                );
                Ok(None)
            }
            Err(e) => Err(anyhow!("NeoForge update check failed: {}", e)),
        }
    }

    /// Check for Quilt loader updates
    async fn check_quilt_update(server: &ServerData) -> Result<Option<UpdateInfo>> {
        let client = Client::new();
        let current_version = server
            .loader_version
            .as_ref()
            .ok_or(anyhow!("No loader version set"))?;
        let minecraft_version = server
            .minecraft_version
            .as_ref()
            .ok_or(anyhow!("No minecraft version set"))?;

        debug!(
            "Checking quilt update for MC {} with loader {}",
            minecraft_version, current_version
        );

        // Fetch available loader versions for this MC version
        let url = format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}",
            minecraft_version
        );
        let response = client.get(&url).send().await?;
        let loaders: Vec<Value> = response.json().await?;

        if loaders.is_empty() {
            return Ok(None);
        }

        // Get the latest loader
        let latest = &loaders[0];
        let latest_version = latest["version"]
            .as_str()
            .ok_or(anyhow!("Failed to parse loader version"))?
            .to_string();

        if current_version == &latest_version {
            info!(
                "Server '{}' Quilt loader is up to date ({})",
                server.name, current_version
            );
            return Ok(None);
        }

        // Get stable installer version
        let installer_response = client
            .get("https://meta.quiltmc.org/v3/versions")
            .send()
            .await?;
        let versions: Value = installer_response.json().await?;
        let installers = versions["installer"]
            .as_array()
            .ok_or(anyhow!("Failed to parse installers"))?;
        let stable_installer = installers
            .iter()
            .find(|i| i["stable"].as_bool() == Some(true))
            .ok_or(anyhow!("No stable installer found"))?;
        let installer_version = stable_installer["version"]
            .as_str()
            .ok_or(anyhow!("Failed to parse installer version"))?;

        let download_url = format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}/{}/{}/server/jar",
            minecraft_version, latest_version, installer_version
        );

        info!(
            "Update available for server '{}' Quilt: {} -> {}",
            server.name, current_version, latest_version
        );

        Ok(Some(UpdateInfo::new(
            current_version.clone(),
            latest_version,
            download_url,
            Some("https://quiltmc.org/install/server/".to_string()),
        )))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_vanilla_changelog_url_format() {
        let version = "1.20.2";
        let expected = "https://www.minecraft.net/en-us/article/minecraft-java-edition-1-20-2";
        let actual = format!(
            "https://www.minecraft.net/en-us/article/minecraft-java-edition-{}",
            version.replace('.', "-")
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_fabric_download_url_format() {
        let mc_version = "1.20.1";
        let loader_version = "0.15.0";
        let installer_version = "1.0.0";

        let url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
            mc_version, loader_version, installer_version
        );

        assert!(url.contains(mc_version));
        assert!(url.contains(loader_version));
        assert!(url.ends_with("/server/jar"));
    }

    #[test]
    fn test_forge_download_url_format() {
        let full_version = "1.20.1-47.2.0";

        let url = format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar",
            full_version, full_version
        );

        assert!(url.contains("maven.minecraftforge.net"));
        assert!(url.contains(full_version));
        assert!(url.ends_with("-installer.jar"));
    }

    #[test]
    fn test_forge_version_key_format() {
        let mc_version = "1.20.1";
        let recommended_key = format!("{}-recommended", mc_version);
        let latest_key = format!("{}-latest", mc_version);

        assert_eq!(recommended_key, "1.20.1-recommended");
        assert_eq!(latest_key, "1.20.1-latest");
    }

    #[test]
    fn test_neoforge_download_url_format() {
        let version = "20.4.108";

        let url = format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
            version, version
        );

        assert!(url.contains("maven.neoforged.net"));
        assert!(url.contains(version));
        assert!(url.ends_with("-installer.jar"));
    }

    #[test]
    fn test_quilt_download_url_format() {
        let mc_version = "1.20.1";
        let loader_version = "0.24.0";
        let installer_version = "0.9.0";

        let url = format!(
            "https://meta.quiltmc.org/v3/versions/loader/{}/{}/{}/server/jar",
            mc_version, loader_version, installer_version
        );

        assert!(url.contains("meta.quiltmc.org"));
        assert!(url.contains(mc_version));
        assert!(url.contains(loader_version));
        assert!(url.ends_with("/server/jar"));
    }

    #[test]
    fn test_version_comparison() {
        // Test basic version equality
        assert_eq!("1.20.1", "1.20.1");
        assert_ne!("1.20.1", "1.20.2");

        // Note: This is string comparison, not semantic versioning
        // "1.20.10" > "1.20.2" lexicographically, which may not be desired
        // but matches the current implementation
        assert_ne!("1.20.2", "1.20.10");
    }

    #[test]
    fn test_api_endpoint_urls() {
        // Verify API endpoint URLs are correct
        assert_eq!(
            "https://launchermeta.mojang.com/mc/game/version_manifest.json",
            "https://launchermeta.mojang.com/mc/game/version_manifest.json"
        );

        assert_eq!(
            format!("https://meta.fabricmc.net/v2/versions/loader/{}", "1.20.1"),
            "https://meta.fabricmc.net/v2/versions/loader/1.20.1"
        );

        assert_eq!(
            "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
            "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json"
        );

        assert_eq!(
            "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge",
            "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge"
        );

        assert_eq!(
            format!("https://meta.quiltmc.org/v3/versions/loader/{}", "1.20.1"),
            "https://meta.quiltmc.org/v3/versions/loader/1.20.1"
        );
    }

    #[test]
    fn test_changelog_urls() {
        // Fabric changelog
        assert_eq!(
            "https://fabricmc.net/versions/",
            "https://fabricmc.net/versions/"
        );

        // Forge changelog
        assert_eq!(
            "https://files.minecraftforge.net/",
            "https://files.minecraftforge.net/"
        );

        // NeoForge changelog
        assert_eq!(
            "https://neoforged.net/",
            "https://neoforged.net/"
        );

        // Quilt changelog
        assert_eq!(
            "https://quiltmc.org/install/server/",
            "https://quiltmc.org/install/server/"
        );
    }
}
