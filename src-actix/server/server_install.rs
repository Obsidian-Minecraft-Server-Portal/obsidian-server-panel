use crate::broadcast;
use crate::broadcast::broadcast_data::BroadcastMessage;
use crate::server::server_data::ServerData;
use crate::server::server_type::ServerType;
use anyhow::{Result, anyhow, bail, ensure};
use log::info;
use std::path::Path;

impl ServerData {
    /// Whether the configured server jar exists inside the server directory.
    pub(crate) fn has_launchable_jar(&self) -> bool {
        !self.server_jar.is_empty() && self.get_directory_path().join(&self.server_jar).is_file()
    }

    /// Resolves the configured Java executable the way the spawn will: absolute paths and
    /// bare names are used as-is (bare names go through `PATH`), relative paths are tried
    /// against the server directory first and then against the panel's working directory,
    /// where panel-managed JDKs live (e.g. `./meta/java/gamma-17.0.15/bin/java`).
    pub(crate) fn resolve_java_executable(java_executable: &str, working_dir: &Path) -> String {
        let is_pathy = java_executable.contains('/') || java_executable.contains('\\');
        if !is_pathy || Path::new(java_executable).is_absolute() {
            return java_executable.to_string();
        }
        if working_dir.join(java_executable).is_file() {
            return working_dir.join(java_executable).to_string_lossy().into_owned();
        }
        if let Ok(panel_dir) = std::env::current_dir()
            && panel_dir.join(java_executable).is_file()
        {
            return panel_dir.join(java_executable).to_string_lossy().into_owned();
        }
        java_executable.to_string()
    }

    /// Checks that the configured Java executable can actually be spawned, mirroring how
    /// `tokio::process::Command` resolves it: a value containing a path separator is
    /// resolved against the child's working directory (the server directory), a bare name
    /// is looked up on `PATH`. The value is used exactly as configured, like the spawn does.
    pub(crate) fn validate_java_executable(java_executable: &str, working_dir: &Path) -> Result<()> {
        ensure!(!java_executable.trim().is_empty(), "No Java executable is configured for this server; pick one in the server options");
        if java_executable.contains('/') || java_executable.contains('\\') {
            let resolved = Self::resolve_java_executable(java_executable, working_dir);
            let path = Path::new(&resolved);
            ensure!(path.is_file(), "Java executable '{java_executable}' is not a file");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(path)?.permissions().mode();
                ensure!(mode & 0o111 != 0, "Java executable '{java_executable}' is not executable");
            }
            return Ok(());
        }
        let path_var = std::env::var("PATH").unwrap_or_default();
        if std::env::split_paths(&path_var).any(|dir| dir.join(java_executable).is_file()) {
            return Ok(());
        }
        bail!("Java executable '{java_executable}' was not found on PATH");
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

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn java_validation_rejects_missing_paths() {
        let dir = temp_dir("osp-java-val");
        assert!(ServerData::validate_java_executable("", &dir).is_err());
        assert!(ServerData::validate_java_executable("   ", &dir).is_err());
        assert!(ServerData::validate_java_executable("/nonexistent/java-xyz", &dir).is_err());
        assert!(ServerData::validate_java_executable("definitely-not-on-path-xyz", &dir).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn java_validation_accepts_resolvable_executables() {
        let dir = temp_dir("osp-java-val");
        assert!(ServerData::validate_java_executable("/bin/sh", &dir).is_ok());
        assert!(ServerData::validate_java_executable("sh", &dir).is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn java_validation_rejects_non_executable_files() {
        let dir = temp_dir("osp-java-val");
        let path = dir.join("java");
        std::fs::write(&path, b"").unwrap();
        assert!(ServerData::validate_java_executable(path.to_str().unwrap(), &dir).is_err());
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(ServerData::validate_java_executable(path.to_str().unwrap(), &dir).is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[cfg(unix)]
    fn java_validation_resolves_relative_paths_against_the_server_directory() {
        let dir = temp_dir("osp-java-val");
        std::fs::write(dir.join("jre"), b"").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.join("jre"), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(ServerData::validate_java_executable("jre", &dir).is_err());
        assert!(ServerData::validate_java_executable("./jre", &dir).is_ok());
        assert!(ServerData::validate_java_executable("./jre", &temp_dir("osp-java-val-empty")).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn java_validation_does_not_trim_the_value() {
        let dir = temp_dir("osp-java-val");
        assert!(ServerData::validate_java_executable(" sh ", &dir).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
