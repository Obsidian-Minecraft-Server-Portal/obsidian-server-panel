use anyhow::Result;
use std::path::PathBuf;
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerProperties {
    #[serde(rename = "server-port")]
    pub server_port: Option<i64>,
}

impl ServerProperties {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let file_content = std::fs::read_to_string(path.into())?;
        let properties: Self = serde_ini::from_str(&file_content)?;
        Ok(properties)
    }
}
