use thiserror::Error;

#[derive(Error, Debug)]
pub enum FabricError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Fabric API returned error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("No stable installer version found")]
    NoStableInstaller,

    #[error("No loader versions available for Minecraft {mc_version}")]
    NoLoaderVersions { mc_version: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, FabricError>;
