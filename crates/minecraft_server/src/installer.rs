use crate::error::McServerError;
use crate::events::{ServerEvent, ServerEventHandler};
use crate::models::{ServerConfig, ServerType};
use crate::Result;

#[cfg(feature = "logging")]
use log::{debug, info};

/// Install a vanilla Minecraft server by downloading the server JAR via piston-mc.
pub async fn install_vanilla(
    config: &ServerConfig,
    handler: &impl ServerEventHandler,
) -> Result<String> {
    handler
        .on_event(ServerEvent::InstallProgress {
            file: "server.jar".to_string(),
            completed: false,
            total: 1,
            current: 0,
        })
        .await;

    #[cfg(feature = "logging")]
    info!(
        "Downloading vanilla server JAR for Minecraft {}",
        config.minecraft_version
    );

    let manifest = piston_mc::manifest_v2::ManifestV2::fetch()
        .await
        .map_err(McServerError::Other)?;

    let version = manifest
        .version(&config.minecraft_version)
        .await
        .map_err(McServerError::Other)?
        .ok_or_else(|| McServerError::VersionNotFound(config.minecraft_version.clone()))?;

    if version.downloads.server.is_none() {
        return Err(McServerError::NoServerDownload(
            config.minecraft_version.clone(),
        ));
    }

    // Ensure the server directory exists
    tokio::fs::create_dir_all(&config.directory).await?;

    let jar_name = format!("minecraft_server_{}.jar", config.minecraft_version);
    let jar_path = config.directory.join(&jar_name);

    version
        .download_server(&jar_path, true, None)
        .await
        .map_err(McServerError::Other)?;

    #[cfg(feature = "logging")]
    info!("Server JAR downloaded to {}", jar_path.display());

    // Accept EULA
    crate::eula::accept_eula(&config.directory)?;

    handler
        .on_event(ServerEvent::InstallProgress {
            file: "server.jar".to_string(),
            completed: true,
            total: 1,
            current: 1,
        })
        .await;

    Ok(jar_name)
}

/// Install a Fabric server using the fabric-loader crate.
pub async fn install_fabric(config: &ServerConfig) -> Result<InstallResult> {
    #[cfg(feature = "logging")]
    info!(
        "Installing Fabric server for MC {} with loader {}",
        config.minecraft_version,
        config.loader_version.as_deref().unwrap_or("latest")
    );

    let client = fabric_loader::FabricClient::new();

    // If no loader version specified, get the latest
    let loader_version = match &config.loader_version {
        Some(v) => v.clone(),
        None => {
            let loaders = client
                .get_loader_versions(&config.minecraft_version)
                .await
                .map_err(|e| McServerError::InstallFailed(e.to_string()))?;
            loaders
                .first()
                .ok_or_else(|| {
                    McServerError::InstallFailed(
                        "No Fabric loader versions available".to_string(),
                    )
                })?
                .loader
                .version
                .clone()
        }
    };

    let result = client
        .install_server(
            &config.minecraft_version,
            &loader_version,
            &config.directory,
            None,
        )
        .await
        .map_err(|e| McServerError::InstallFailed(e.to_string()))?;

    // Accept EULA
    crate::eula::accept_eula(&config.directory)?;

    Ok(InstallResult {
        server_jar: result
            .server_jar
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        java_args: String::new(),
        loader_version: Some(loader_version),
    })
}

/// Install a Forge server using the forge-loader crate.
pub async fn install_forge(config: &ServerConfig) -> Result<InstallResult> {
    let forge_version = config
        .loader_version
        .as_deref()
        .ok_or_else(|| McServerError::InvalidConfig("Forge requires a loader version".into()))?;

    #[cfg(feature = "logging")]
    info!(
        "Installing Forge server for MC {} with Forge {}",
        config.minecraft_version, forge_version
    );

    let client = forge_loader::ForgeClient::new();
    let result = client
        .install_server(forge_loader::ForgeInstallOptions {
            mc_version: &config.minecraft_version,
            forge_version,
            install_dir: &config.directory,
            java_executable: &config.java_executable,
            download_progress: None,
        })
        .await
        .map_err(|e| McServerError::InstallFailed(e.to_string()))?;

    // Accept EULA
    crate::eula::accept_eula(&config.directory)?;

    Ok(InstallResult {
        server_jar: result.server_jar,
        java_args: result.java_args,
        loader_version: Some(forge_version.to_string()),
    })
}

/// Install a NeoForge server using the neoforge-loader crate.
pub async fn install_neoforge(config: &ServerConfig) -> Result<InstallResult> {
    let neoforge_version = config
        .loader_version
        .as_deref()
        .ok_or_else(|| {
            McServerError::InvalidConfig("NeoForge requires a loader version".into())
        })?;

    #[cfg(feature = "logging")]
    info!(
        "Installing NeoForge server with version {}",
        neoforge_version
    );

    let client = neoforge_loader::NeoForgeClient::new();
    let result = client
        .install_server(neoforge_loader::NeoForgeInstallOptions {
            neoforge_version,
            install_dir: &config.directory,
            java_executable: &config.java_executable,
            download_progress: None,
        })
        .await
        .map_err(|e| McServerError::InstallFailed(e.to_string()))?;

    // Accept EULA
    crate::eula::accept_eula(&config.directory)?;

    Ok(InstallResult {
        server_jar: result.server_jar,
        java_args: result.java_args,
        loader_version: Some(neoforge_version.to_string()),
    })
}

/// Result of a server installation.
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Server JAR filename to use for launching.
    pub server_jar: String,
    /// Extra Java arguments required by the loader (e.g. Forge's `@libraries/...`).
    pub java_args: String,
    /// Resolved loader version.
    pub loader_version: Option<String>,
}

/// Install a server based on the configuration's server type.
/// Updates `config.server_jar` and `config.java_args` with the installation results.
pub async fn install_server(
    config: &mut ServerConfig,
    handler: &impl ServerEventHandler,
) -> Result<()> {
    // Ensure the server directory exists
    tokio::fs::create_dir_all(&config.directory).await?;

    match config.server_type {
        ServerType::Vanilla => {
            let jar_name = install_vanilla(config, handler).await?;
            config.server_jar = jar_name;
        }
        ServerType::Fabric => {
            let result = install_fabric(config).await?;
            config.server_jar = result.server_jar;
            if let Some(v) = result.loader_version {
                config.loader_version = Some(v);
            }
        }
        ServerType::Forge => {
            let result = install_forge(config).await?;
            config.server_jar = result.server_jar;
            config.java_args = result.java_args;
        }
        ServerType::NeoForge => {
            let result = install_neoforge(config).await?;
            config.server_jar = result.server_jar;
            config.java_args = result.java_args;
        }
        ServerType::Custom => {
            // Custom servers bring their own JAR - nothing to install
            #[cfg(feature = "logging")]
            debug!("Custom server type - skipping installation");
        }
        ServerType::Quilt => {
            return Err(McServerError::InstallFailed(
                "Quilt server installation is not yet supported".to_string(),
            ));
        }
    }

    Ok(())
}
