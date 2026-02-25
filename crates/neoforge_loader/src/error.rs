use thiserror::Error;

#[derive(Error, Debug)]
pub enum NeoForgeError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("NeoForge API returned error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("No NeoForge versions found")]
    NoVersions,

    #[error("No NeoForge version found for Minecraft {mc_version}")]
    NoVersionForMc { mc_version: String },

    #[error("Java executable not found or not executable: {path}")]
    JavaNotFound { path: String },

    #[error("NeoForge installer failed with exit code {exit_code}")]
    InstallerFailed { exit_code: i32 },

    #[error("Failed to parse start script: {reason}")]
    ScriptParseError { reason: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, NeoForgeError>;
