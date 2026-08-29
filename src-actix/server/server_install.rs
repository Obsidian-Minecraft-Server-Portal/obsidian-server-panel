use crate::broadcast;
use crate::broadcast::broadcast_data::BroadcastMessage;
use crate::server::server_data::ServerData;
use crate::server::server_type::ServerType;
use anyhow::{anyhow, bail, ensure, Result};
use log::info;
use std::path::Path;

impl ServerData {
    /// Whether the configured server jar exists inside the server directory.
    pub(crate) fn has_launchable_jar(&self) -> bool {
        !self.server_jar.is_empty() && self.get_directory_path().join(&self.server_jar).is_file()
    }

    /// Checks that the configured Java executable can actually be spawned, mirroring how
    /// `tokio::process::Command` resolves it: a value containing a path separator is used
    /// as-is, a bare name is looked up on `PATH`.
    pub(crate) fn validate_java_executable(java_executable: &str) -> Result<()> {
        let java = java_executable.trim();
        ensure!(!java.is_empty(), "No Java executable is configured for this server; pick one in the server options");
        if java.contains('/') || java.contains('\\') {
            let path = Path::new(java);
            ensure!(path.is_file(), "Java executable '{java}' is not a file");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(path)?.permissions().mode();
                ensure!(mode & 0o111 != 0, "Java executable '{java}' is not executable");
            }
            return Ok(());
        }
        let path_var = std::env::var("PATH").unwrap_or_default();
        if std::env::split_paths(&path_var).any(|dir| dir.join(java).is_file()) {
            return Ok(());
        }
        bail!("Java executable '{java}' was not found on PATH");
    }

    /// Places a launchable server jar in the server directory, downloading the artifact for
    /// the configured loader when the directory has none. Servers that already ship a jar
    /// (uploaded through the UI or installed by a modpack) are left untouched.
    pub(crate) async fn download_server_artifact(&mut self) -> Result<()> {
        let directory = self.get_directory_path();
        if let Some(jar) = super::modpack_install::detect_server_jar(&directory)? {
            info!("Using existing server jar '{}' for server '{}'", jar, self.name);
            self.server_jar = jar;
            self.save().await?;
            return Ok(());
        }
        match self.server_type {
            Some(ServerType::Vanilla) => self.install_vanilla_jar().await,
            Some(ServerType::Fabric) => self.install_fabric_jar().await,
            Some(ServerType::Spigot | ServerType::CraftBukkit) => self.install_getbukkit_jar().await,
            Some(ServerType::Quilt) => bail!("Quilt server jars cannot be downloaded automatically; place one in {}", directory.display()),
            _ => bail!("No server jar found in {} - install one from the Content tab or set one in the server options", directory.display()),
        }
    }

    async fn install_vanilla_jar(&mut self) -> Result<()> {
        let version = self.minecraft_version.clone().ok_or_else(|| anyhow!("Vanilla server '{}' has no Minecraft version set", self.name))?;
        info!("Downloading the vanilla {version} server jar for server '{}'", self.name);
        let config = self.to_server_config();
        let jar = minecraft_server::installer::install_vanilla(&config, &minecraft_server::NoOpHandler)
            .await
            .map_err(|e| anyhow!("Failed to download the vanilla {version} server jar: {e}"))?;
        self.server_jar = jar;
        self.save().await?;
        broadcast::broadcast(BroadcastMessage::ServerUpdate { server: self.clone() });
        Ok(())
    }

    async fn install_fabric_jar(&mut self) -> Result<()> {
        let version = self.minecraft_version.clone().ok_or_else(|| anyhow!("Fabric server '{}' has no Minecraft version set", self.name))?;
        info!("Downloading the Fabric {version} server launcher for server '{}'", self.name);
        let config = self.to_server_config();
        let result = minecraft_server::installer::install_fabric(&config)
            .await
            .map_err(|e| anyhow!("Failed to download the Fabric {version} server launcher: {e}"))?;
        self.server_jar = result.server_jar;
        if let Some(loader_version) = result.loader_version {
            self.loader_version = Some(loader_version);
        }
        self.save().await?;
        broadcast::broadcast(BroadcastMessage::ServerUpdate { server: self.clone() });
        Ok(())
    }

    async fn install_getbukkit_jar(&mut self) -> Result<()> {
        let flavor = match self.server_type {
            Some(ServerType::Spigot) => "spigot",
            Some(ServerType::CraftBukkit) => "craftbukkit",
            _ => bail!("Only Spigot and CraftBukkit servers can be downloaded from getbukkit"),
        };
        let version = self.minecraft_version.clone().ok_or_else(|| anyhow!("{flavor} server '{}' has no Minecraft version set", self.name))?;
        info!("Downloading the {flavor} {version} server jar for server '{}'", self.name);
        let filename = format!("{flavor}-{version}.jar");
        let destination = self.get_directory_path().join(&filename);
        let url = crate::platforms::getbukkit::jar_url(flavor, &version);
        super::modpack_install::download_file(crate::platforms::http_client(), &url, &destination)
            .await
            .map_err(|e| anyhow!("Failed to download the {flavor} {version} server jar: {e}"))?;
        self.server_jar = filename;
        self.save().await?;
        broadcast::broadcast(BroadcastMessage::ServerUpdate { server: self.clone() });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_validation_rejects_missing_paths() {
        assert!(ServerData::validate_java_executable("").is_err());
        assert!(ServerData::validate_java_executable("   ").is_err());
        assert!(ServerData::validate_java_executable("/nonexistent/java-xyz").is_err());
        assert!(ServerData::validate_java_executable("definitely-not-on-path-xyz").is_err());
    }

    #[test]
    #[cfg(unix)]
    fn java_validation_accepts_resolvable_executables() {
        assert!(ServerData::validate_java_executable("/bin/sh").is_ok());
        assert!(ServerData::validate_java_executable(" sh ").is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn java_validation_rejects_non_executable_files() {
        let dir = std::env::temp_dir().join(format!("osp-java-val-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("java");
        std::fs::write(&path, b"").unwrap();
        assert!(ServerData::validate_java_executable(path.to_str().unwrap()).is_err());
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(ServerData::validate_java_executable(path.to_str().unwrap()).is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }
}
