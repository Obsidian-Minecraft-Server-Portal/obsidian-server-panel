use crate::app_db;
use crate::server::installed_mods::mod_data::ModData;
use crate::server::server_data::ServerData;
use anyhow::Result;
use log::*;
use notify::{RecursiveMode, Watcher};
use std::sync::mpsc;

impl ServerData {
    pub async fn start_watch_server_mod_directory_for_changes(&self) -> Result<()> {
        let (sender, receiver) = mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(sender).map_err(|e| anyhow::anyhow!("Failed to create file watcher: {}", e))?;
        let mods_path = self.get_directory_path().join("mods").canonicalize()?;
        watcher.watch(&mods_path, RecursiveMode::NonRecursive)?;

        debug!("Started watching mods directory for server {}: {:?}", self.name, mods_path);

        let pool = app_db::open_pool().await?;
        for res in receiver {
            match res {
                Ok(event) => {
                    debug!("File watcher event for server {}: {:?}", self.name, event);
                    if let Err(e) = self.on_file_watcher_trigger(&event, &pool).await {
                        error!("Error processing file watcher event for server {}: {}", self.name, e);
                    }
                }
                Err(e) => {
                    error!("File watcher error for server {}: {}", self.name, e);
                    continue;
                }
            }
        }

        pool.close().await;
        Ok(())
    }

    async fn on_file_watcher_trigger(&self, event: &notify::Event, pool: &sqlx::MySqlPool) -> Result<()> {
        let paths = &event.paths;
        match &event.kind {
            notify::EventKind::Create(_) => {
                debug!("File created in mods directory for server {}: {:?}", self.name, event.paths);
                for path in paths {
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            let extension: String = extension.to_string_lossy().into();
                            if extension != "jar" {
                                return Ok(());
                            }
                        }
                        let exists = if let Some(filename) = path.file_name() {
                            let count: i64 = sqlx::query_scalar(r#"select count(*) from installed_mods where filename = ? and server_id = ?"#)
                                .bind(filename.to_string_lossy())
                                .bind(self.id as u32)
                                .fetch_one(pool)
                                .await?;
                            count > 0
                        } else {
                            false
                        };
                        if exists {
                            return Ok(());
                        }
                        match ModData::from_path(&path).await {
                            Ok(Some(mod_data)) => {
                                if let Err(e) = self.insert_installed_mod(&mod_data, pool).await {
                                    error!("Failed to insert mod for server {}: {}", self.name, e);
                                }
                            }
                            Ok(None) => {
                                debug!("No mod data found for file: {:?}", path);
                            }
                            Err(e) => {
                                error!("Failed to parse mod data from {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
            notify::EventKind::Modify(_) => {
                debug!("File modified in mods directory for server {}: {:?}", self.name, event.paths);
                for path in paths {
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            let extension: String = extension.to_string_lossy().into();
                            if extension != "jar" {
                                return Ok(());
                            }
                        }
                        let exists = if let Some(filename) = path.file_name() {
                            let count: i64 = sqlx::query_scalar(r#"select count(*) from installed_mods where filename = ? and server_id = ?"#)
                                .bind(filename.to_string_lossy())
                                .bind(self.id as u32)
                                .fetch_one(pool)
                                .await?;
                            count > 0
                        } else {
                            false
                        };
                        if exists {
                            return Ok(());
                        }
                        // Delete existing entry first
                        if let Some(filename) = path.file_name() {
                            let filename_str = filename.to_string_lossy();
                            if let Err(e) = self.delete_installed_mod(&filename_str, pool).await {
                                error!("Failed to delete mod entry for server {}: {}", self.name, e);
                            }
                        }

                        // Re-add the modified file
                        match ModData::from_path(&path).await {
                            Ok(Some(mod_data)) => {
                                if let Err(e) = self.insert_installed_mod(&mod_data, pool).await {
                                    error!("Failed to re-insert modified mod for server {}: {}", self.name, e);
                                }
                            }
                            Ok(None) => {
                                debug!("No mod data found for modified file: {:?}", path);
                            }
                            Err(e) => {
                                error!("Failed to parse mod data from modified file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                debug!("File removed from mods directory for server {}: {:?}", self.name, event.paths);
                for path in paths {
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy();
                        if let Err(e) = self.delete_installed_mod(&filename_str, pool).await {
                            error!("Failed to delete removed mod for server {}: {}", self.name, e);
                        }
                    }
                }
            }
            _ => {
                trace!("Other file event for server {}: {:?}", self.name, event);
            }
        }

        Ok(())
    }
}
