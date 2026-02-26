use crate::Result;
use std::path::PathBuf;

/// Parsed server.properties file.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerProperties {
    #[serde(rename = "server-port")]
    pub server_port: Option<i64>,
}

impl ServerProperties {
    /// Load and parse a server.properties file from the given path.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let file_content = std::fs::read_to_string(path.into())?;
        let properties: Self =
            serde_ini::from_str(&file_content).map_err(|e| anyhow::anyhow!("Failed to parse server.properties: {}", e))?;
        Ok(properties)
    }

    /// Load server.properties from a server directory.
    pub fn load_from_dir(server_dir: &std::path::Path) -> Result<Self> {
        Self::load(server_dir.join("server.properties"))
    }
}
