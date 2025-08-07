use crate::server::server_data::ServerData;
use crate::server::backups::backup_data::BackupData;
use crate::server::backups::backup_type::BackupType;
use crate::{app_db};
use anyhow::Result;
use std::fs::File;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;
use zip::{ZipWriter, write::FileOptions};

impl ServerData {
    pub async fn backup(&self) -> Result<(), String> {
        self.create_backup_with_description(None).await
    }

    pub async fn create_backup_with_description(&self, description: Option<String>) -> Result<(), String> {
        log::info!("Starting backup for server: {}", self.name);

        // Create backup directory if it doesn't exist
        let backup_dir = PathBuf::from("./backups").join(&self.name);
        if let Err(e) = fs::create_dir_all(&backup_dir).await {
            return Err(format!("Failed to create backup directory: {}", e));
        }

        // Generate backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_filename = format!("backup_{}_{}.zip", self.backup_type, timestamp);
        let backup_path = backup_dir.join(&backup_filename);

        // Create the backup archive
        let file_size = match self.create_backup_archive(&backup_path).await {
            Ok(size) => size,
            Err(e) => return Err(format!("Failed to create backup archive: {}", e)),
        };

        // Save backup metadata to database
        let pool = match app_db::open_pool().await {
            Ok(pool) => pool,
            Err(e) => return Err(format!("Failed to open database: {}", e)),
        };

        let mut backup_data = BackupData::new(
            self.id,
            backup_filename,
            self.backup_type.clone(),
            file_size,
            description,
        );

        if let Err(e) = backup_data.create(&pool).await {
            pool.close().await;
            return Err(format!("Failed to save backup metadata: {}", e));
        }

        // Apply backup retention
        if let Err(e) = self.apply_backup_retention(&pool).await {
            log::warn!("Failed to apply backup retention for server {}: {}", self.name, e);
        }

        pool.close().await;
        log::info!("Backup completed successfully for server: {}", self.name);
        Ok(())
    }

    async fn create_backup_archive(&self, backup_path: &Path) -> Result<i64> {
        let file = File::create(backup_path)?;
        let mut zip = ZipWriter::new(file);

        // Use store method (no compression) as requested
        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored);

        let source_path = match self.backup_type {
            BackupType::World => {
                // For world backups, only backup the world directory
                let properties = self.get_server_properties()
                    .map_err(|e| anyhow::anyhow!("Failed to read server properties: {}", e))?;
                let level_name = properties.level_name.unwrap_or_else(|| "world".to_string());
                self.get_directory_path().join(level_name)
            }
            BackupType::Full | BackupType::Incremental => {
                // For full and incremental backups, backup the entire server directory
                // Note: For now, we'll treat incremental the same as full
                // TODO: Implement proper incremental backup logic
                self.get_directory_path()
            }
        };

        if !source_path.exists() {
            return Err(anyhow::anyhow!("Source path does not exist: {:?}", source_path));
        }

        // Add files to zip archive
        for entry in WalkDir::new(&source_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(&source_path)?;

            // Skip empty directory names and backup files
            if name.as_os_str().is_empty() ||
                path.extension().is_some_and(|ext| ext == "zip" || ext == "backup") {
                continue;
            }

            if path.is_file() {
                zip.start_file(name.to_string_lossy().replace('\\', "/"), options)?;
                let file_content = std::fs::read(path)?;
                std::io::copy(&mut file_content.as_slice(), &mut zip)?;
            } else if path.is_dir() {
                zip.add_directory(name.to_string_lossy().replace('\\', "/"), options)?;
            }
        }

        zip.finish()?;

        // Get file size
        let metadata = std::fs::metadata(backup_path)?;
        Ok(metadata.len() as i64)
    }

    async fn apply_backup_retention(&self, pool: &sqlx::SqlitePool) -> Result<()> {
        let backups = BackupData::list_by_server(self.id as i64, pool).await?;
        
        if backups.len() > self.backup_retention as usize {
            let backups_to_delete = &backups[self.backup_retention as usize..];
            
            for backup in backups_to_delete {
                // Delete the backup file
                let backup_path = backup.get_file_path(&self.name);
                if backup_path.exists() {
                    if let Err(e) = fs::remove_file(&backup_path).await {
                        log::warn!("Failed to delete backup file {:?}: {}", backup_path, e);
                    }
                }
                
                // Delete from database
                if let Err(e) = backup.delete(pool).await {
                    log::warn!("Failed to delete backup metadata for {}: {}", backup.filename, e);
                }
            }
            
            log::info!("Applied backup retention for server {}: deleted {} old backups", 
                      self.name, backups_to_delete.len());
        }
        
        Ok(())
    }

    pub async fn list_backups(&self) -> Result<Vec<BackupData>, String> {
        let pool = match app_db::open_pool().await {
            Ok(pool) => pool,
            Err(e) => return Err(format!("Failed to open database: {}", e)),
        };

        let backups = match BackupData::list_by_server(self.id as i64, &pool).await {
            Ok(backups) => backups,
            Err(e) => {
                pool.close().await;
                return Err(format!("Failed to list backups: {}", e));
            }
        };

        pool.close().await;
        Ok(backups)
    }

    pub async fn delete_backup(&self, backup_id: i64) -> Result<(), String> {
        let pool = match app_db::open_pool().await {
            Ok(pool) => pool,
            Err(e) => return Err(format!("Failed to open database: {}", e)),
        };

        let backup = match BackupData::get_by_id(backup_id, &pool).await {
            Ok(Some(backup)) => backup,
            Ok(None) => {
                pool.close().await;
                return Err("Backup not found".to_string());
            }
            Err(e) => {
                pool.close().await;
                return Err(format!("Failed to get backup: {}", e));
            }
        };

        // Verify the backup belongs to this server
        if backup.server_id != self.id {
            pool.close().await;
            return Err("Backup does not belong to this server".to_string());
        }

        // Delete the backup file
        let backup_path = backup.get_file_path(&self.name);
        if backup_path.exists() {
            if let Err(e) = fs::remove_file(&backup_path).await {
                log::warn!("Failed to delete backup file {:?}: {}", backup_path, e);
            }
        }

        // Delete from database
        if let Err(e) = backup.delete(&pool).await {
            pool.close().await;
            return Err(format!("Failed to delete backup metadata: {}", e));
        }

        pool.close().await;
        log::info!("Deleted backup {} for server {}", backup.filename, self.name);
        Ok(())
    }
}