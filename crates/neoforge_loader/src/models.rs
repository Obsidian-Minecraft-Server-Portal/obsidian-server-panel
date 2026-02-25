use serde::{Deserialize, Serialize};

/// NeoForge Maven versions response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoForgeVersionList {
    #[serde(rename = "isSnapshot")]
    pub is_snapshot: bool,
    pub versions: Vec<String>,
}

/// Result of installing a NeoForge server.
#[derive(Debug, Clone)]
pub struct NeoForgeInstallResult {
    /// The server JAR filename (empty string if NeoForge uses `@libraries` style launch).
    pub server_jar: String,
    /// The java arguments extracted from the start script.
    pub java_args: String,
    /// The installer process exit code.
    pub exit_code: i32,
}

/// Options for the NeoForge installation process.
pub struct NeoForgeInstallOptions<'a> {
    /// NeoForge version (e.g. `"21.4.108"`).
    pub neoforge_version: &'a str,
    /// Directory to install into.
    pub install_dir: &'a std::path::Path,
    /// Path to the java executable.
    pub java_executable: &'a str,
    /// Optional progress callback for download: `(bytes_downloaded, total_bytes)`.
    pub download_progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
}

/// Information about an available NeoForge update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoForgeUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub changelog_url: String,
}
