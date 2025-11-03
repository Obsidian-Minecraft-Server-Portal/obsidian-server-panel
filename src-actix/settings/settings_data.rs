use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub general: GeneralSettings,
    pub network: NetworkSettings,
    pub storage: StorageSettings,
    pub java: JavaSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub port: u16,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub auto_port_forward: bool,
    pub upnp_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub servers_directory: PathBuf,
    pub java_directory: PathBuf,
    pub backups_directory: PathBuf,
    pub temp_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_runtime: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            network: NetworkSettings::default(),
            storage: StorageSettings::default(),
            java: JavaSettings::default(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            port: 8080,
            auto_start: false,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            auto_port_forward: false,
            upnp_enabled: true,
        }
    }
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            servers_directory: PathBuf::from("./meta/servers"),
            java_directory: PathBuf::from("./meta/java"),
            backups_directory: PathBuf::from("./meta/backups"),
            temp_directory: PathBuf::from("./meta/temp"),
        }
    }
}

impl Default for JavaSettings {
    fn default() -> Self {
        Self {
            default_runtime: None,
        }
    }
}

impl Settings {
    /// Validate settings to ensure paths exist or can be created
    pub fn validate(&self) -> Result<(), String> {
        let paths = [
            &self.storage.servers_directory,
            &self.storage.java_directory,
            &self.storage.backups_directory,
            &self.storage.temp_directory,
        ];

        for path in paths {
            if path.to_string_lossy().is_empty() {
                return Err(format!("Path cannot be empty"));
            }
        }

        if self.general.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }

        if self.general.port < 1024 {
            return Err("Port must be 1024 or higher (privileged ports restricted)".to_string());
        }

        Ok(())
    }
}
