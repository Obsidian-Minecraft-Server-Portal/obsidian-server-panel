use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerProperties {
    #[serde(rename = "level-name")]
    pub level_name: Option<String>,
    #[serde(rename = "max-players")]
    pub max_players: Option<i64>,
    pub motd: Option<String>,
    #[serde(rename = "query.port")]
    pub query_port: Option<i64>,
    #[serde(rename = "server-port")]
    pub server_port: Option<i64>,
}

impl ServerProperties {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let file_content = std::fs::read_to_string(path.into())?;
        let properties: Self = serde_ini::from_str(&file_content)?;
        Ok(properties)
    }
    pub fn get_custom_property<T>(path: impl Into<PathBuf>, key: impl Into<String>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let file_content = std::fs::read_to_string(path.into())?;
        let properties: HashMap<String, T> = serde_ini::from_str(&file_content)?;
        let value = properties.get(&key.into());
        Ok(value.cloned())
    }
}
