use crate::app_db;
use crate::server::installed_mods::mod_data::ModData;
use crate::server::server_data::ServerData;
use anyhow::Result;
use log::{debug, error, info, trace, warn};
use notify::{Event, EventKind, RecommendedWatcher, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

static WATCHED_SERVERS: OnceLock<Arc<Mutex<HashMap<u64, RecommendedWatcher>>>> = OnceLock::new();

impl ServerData {
    pub async fn start_watch_server_mod_directory_for_changes(&self) -> Result<()> {
        let self_clone = self.clone();
        let server_id = self.id;
        let server_name = self.name.clone();

        // Check if already watching
        {
            let watched_servers = WATCHED_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
            let servers = watched_servers.lock().await;
            if servers.contains_key(&server_id) {
                error!("Server {} is already being watched for mod changes.", server_name);
                return Ok(());
            }
        }

        let mut watcher = notify::recommended_watcher({
            move |event| {
                let self_clone = self_clone.clone();
                let server_name = server_name.clone();
                tokio::spawn(async move {
                    match event {
                        Ok(event) => {
                            debug!("File watcher event for server {}: {:?}", server_name, event);
                            if let Err(e) = self_clone.on_file_watcher_trigger(&event).await {
                                error!(
                                    "Error processing mod file watcher event for server {} on file(s) '{}': {}",
                                    server_name,
                                    event.paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<String>>().join(", "),
                                    e
                                );
                            }
                        }
                        Err(e) => {
                            error!("File watcher error for server {}: {}", server_name, e);
                        }
                    }
                });
            }
        })?;

        let mods_dir = Path::new(&self.directory).join("mods");

        // Ensure directory exists before watching
        if !mods_dir.exists() {
            error!("Mods directory does not exist: {}", mods_dir.display());
            return Err(anyhow::anyhow!("Mods directory does not exist: {}", mods_dir.display()));
        }

        watcher.watch(&mods_dir, notify::RecursiveMode::Recursive)?;

        // Store the watcher
        {
            let watched_servers = WATCHED_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
            let mut servers = watched_servers.lock().await;

            // Double-check in case another thread added it
            if servers.contains_key(&server_id) {
                error!("Server {} is already being watched for mod changes.", self.name);
                return Ok(());
            }

            servers.insert(server_id, watcher);
        }

        info!("Started watching mods directory for server {} at {}", self.name, mods_dir.display());
        Ok(())
    }

    pub async fn stop_watch_server_mod_directory_for_changes(&self) -> Result<()> {
        let watched_servers = WATCHED_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let mut servers = watched_servers.lock().await;

        if servers.remove(&self.id).is_some() {
            warn!("Stopped watching mods directory for server {}", self.name);
        } else {
            error!("Server {} is not being watched for mod changes.", self.name);
        }

        Ok(())
    }

    async fn on_file_watcher_trigger(&self, event: &Event) -> Result<()> {
        match &event.kind {
            EventKind::Create(_) => {
                debug!("File created in mods directory for server {}: {:?}", self.name, event.paths);
                let pool = app_db::open_pool().await?;
                for path in &event.paths {
                    let mod_data = ModData::from_path(path).await?;
                    if let Some(mod_data) = mod_data {
                        self.insert_installed_mod(&mod_data, &pool).await?
                    }
                }
                pool.close().await;
            }
            EventKind::Modify(_) => {
                debug!("File modified in mods directory for server {}: {:?}", self.name, event.paths);
                let pool = app_db::open_pool().await?;
                for path in &event.paths {
                    if let Some(filename) = path.file_name() {
                        self.delete_installed_mod(filename.to_string_lossy().to_string().as_str(), &pool).await?
                    }
                    let mod_data = ModData::from_path(path).await?;
                    if let Some(mod_data) = mod_data {
                        self.insert_installed_mod(&mod_data, &pool).await?
                    }
                }
                pool.close().await;
            }
            EventKind::Remove(_) => {
                debug!("File removed from mods directory for server {}: {:?}", self.name, event.paths);
                let pool = app_db::open_pool().await?;
                for path in &event.paths {
                    if let Some(filename) = path.file_name() {
                        self.delete_installed_mod(filename.to_string_lossy().to_string().as_str(), &pool).await?
                    }
                }
                pool.close().await;
            }
            _ => {
                trace!("Other file event for server {}: {:?}", self.name, event);
            }
        }

        Ok(())
    }
}
