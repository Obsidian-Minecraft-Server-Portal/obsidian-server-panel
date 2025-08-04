mod backup_action;
mod backup_endpoint;
pub mod backup_type;
pub mod backup_data;
pub mod backup_scheduler;

#[cfg(test)]
mod backup_endpoint_tests;

use crate::server::server_data::ServerData;
use anyhow::anyhow;
pub use backup_endpoint::configure;
use std::path::PathBuf;

impl ServerData {
    pub async fn backup_directory(&self) -> anyhow::Result<PathBuf> {
        if self.backup_type == backup_type::BackupType::World {
            let level_name = self.get_server_properties()?.level_name;
            if let Some(level_name) = level_name {
                Ok(self.get_directory_path().join("backups").join(level_name))
            } else {
                Err(anyhow!("Level name is not set"))
            }
        } else {
            Ok(PathBuf::from(format!("./backups/{}", self.name)).canonicalize()?)
        }
    }
}
