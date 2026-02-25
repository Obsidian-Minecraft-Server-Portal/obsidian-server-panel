use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Combined version list from the Fabric Meta API (`/v2/versions/`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricVersionList {
    pub installer: Vec<FabricInstallerVersion>,
    pub loader: Vec<FabricLoaderVersion>,
}

/// A Fabric installer version entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricInstallerVersion {
    pub url: String,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

/// A Fabric loader version entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricLoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

/// Loader info for a specific Minecraft version
/// (from `/v2/versions/loader/{mc_version}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricLoaderInfo {
    pub loader: FabricLoaderVersion,
    pub intermediary: FabricIntermediaryVersion,
}

/// An intermediary mapping version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricIntermediaryVersion {
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

/// Result of installing a Fabric server.
#[derive(Debug, Clone)]
pub struct FabricInstallResult {
    /// Path to the downloaded server JAR file.
    pub server_jar: PathBuf,
}

/// Information about an available Fabric loader update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricUpdateInfo {
    pub current_loader_version: String,
    pub latest_loader_version: String,
    pub download_url: String,
    pub changelog_url: String,
}
