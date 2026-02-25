use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Map of Minecraft version to list of available Forge version strings.
///
/// Example: `{"1.20.1": ["47.3.22", "47.3.21", ...], "1.19.4": [...]}`
pub type ForgeVersionMap = HashMap<String, Vec<String>>;

/// Forge promotions data from `promotions_slim.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgePromotions {
    pub homepage: String,
    pub promos: HashMap<String, String>,
}

/// Result of installing a Forge server.
#[derive(Debug, Clone)]
pub struct ForgeInstallResult {
    /// The server JAR filename (empty string if Forge uses `@libraries` style launch).
    pub server_jar: String,
    /// The java arguments extracted from the start script
    /// (e.g. `@libraries/net/minecraftforge/forge/...`).
    pub java_args: String,
    /// The installer process exit code.
    pub exit_code: i32,
}

/// Options for the Forge installation process.
pub struct ForgeInstallOptions<'a> {
    /// Minecraft version (e.g. `"1.20.1"`).
    pub mc_version: &'a str,
    /// Forge version (e.g. `"47.3.22"`).
    pub forge_version: &'a str,
    /// Directory to install into.
    pub install_dir: &'a std::path::Path,
    /// Path to the java executable.
    pub java_executable: &'a str,
    /// Optional progress callback for download: `(bytes_downloaded, total_bytes)`.
    pub download_progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
}

/// Information about an available Forge update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub changelog_url: String,
    /// Whether the latest version is a "recommended" build (vs "latest").
    pub is_recommended: bool,
}
