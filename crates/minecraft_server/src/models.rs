use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

/// Server loader type.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ServerType {
    Vanilla,
    Forge,
    Fabric,
    NeoForge,
    Quilt,
    Custom,
}

impl From<u8> for ServerType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Vanilla,
            1 => Self::Forge,
            2 => Self::Fabric,
            3 => Self::NeoForge,
            4 => Self::Quilt,
            _ => Self::Custom,
        }
    }
}

impl Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Vanilla => "Vanilla",
                Self::Forge => "Forge",
                Self::Fabric => "Fabric",
                Self::NeoForge => "NeoForge",
                Self::Quilt => "Quilt",
                Self::Custom => "Custom",
            }
        )
    }
}

impl FromStr for ServerType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vanilla" => Ok(Self::Vanilla),
            "forge" => Ok(Self::Forge),
            "fabric" => Ok(Self::Fabric),
            "neo-forge" | "neoforge" => Ok(Self::NeoForge),
            "quilt" => Ok(Self::Quilt),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Unknown server type: {}", s)),
        }
    }
}

impl<'de> Deserialize<'de> for ServerType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ServerType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Server lifecycle status.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ServerStatus {
    Idle,
    Running,
    Stopped,
    Error,
    Starting,
    Stopping,
    Crashed,
    Hanging,
}

impl Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Idle => "idle",
                Self::Running => "running",
                Self::Stopped => "stopped",
                Self::Error => "error",
                Self::Starting => "starting",
                Self::Stopping => "stopping",
                Self::Crashed => "crashed",
                Self::Hanging => "hanging",
            }
        )
    }
}

impl FromStr for ServerStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "idle" => Ok(Self::Idle),
            "running" => Ok(Self::Running),
            "stopped" => Ok(Self::Stopped),
            "error" => Ok(Self::Error),
            "starting" => Ok(Self::Starting),
            "stopping" => Ok(Self::Stopping),
            "crashed" => Ok(Self::Crashed),
            "hanging" => Ok(Self::Hanging),
            _ => Err(format!("Invalid server status: {}", s)),
        }
    }
}

impl<'de> Deserialize<'de> for ServerStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ServerStatus::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Database-free server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Human-readable server name.
    pub name: String,
    /// Absolute path to the server directory.
    pub directory: PathBuf,
    /// Path to the Java executable (e.g. "java" or "/usr/bin/java").
    pub java_executable: String,
    /// Extra JVM arguments (not including -Xmx/-Xms).
    pub java_args: String,
    /// Maximum heap memory in GB (-Xmx).
    pub max_memory_gb: u8,
    /// Minimum heap memory in GB (-Xms).
    pub min_memory_gb: u8,
    /// Extra Minecraft server arguments.
    pub minecraft_args: String,
    /// Server JAR filename (e.g. "server.jar").
    pub server_jar: String,
    /// Minecraft version (e.g. "1.21.4").
    pub minecraft_version: String,
    /// Server loader type.
    pub server_type: ServerType,
    /// Loader version (e.g. "0.15.0" for Fabric, "47.3.22" for Forge).
    pub loader_version: Option<String>,
}

impl ServerConfig {
    /// Save the configuration to a JSON file.
    pub fn save(&self, path: &std::path::Path) -> crate::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| crate::McServerError::Other(e.into()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a configuration from a JSON file.
    pub fn load(path: &std::path::Path) -> crate::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let config: Self =
            serde_json::from_str(&json).map_err(|e| crate::McServerError::Other(e.into()))?;
        Ok(config)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "Minecraft Server".to_string(),
            directory: PathBuf::from("."),
            java_executable: "java".to_string(),
            java_args: String::new(),
            max_memory_gb: 2,
            min_memory_gb: 1,
            minecraft_args: String::new(),
            server_jar: "server.jar".to_string(),
            minecraft_version: String::new(),
            server_type: ServerType::Vanilla,
            loader_version: None,
        }
    }
}

/// Information about a running or configured server.
#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub config: ServerConfig,
    pub status: ServerStatus,
    pub pid: Option<u32>,
}
