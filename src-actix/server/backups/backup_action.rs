use crate::app_db;
use crate::server::backups::backup_data::BackupData;
use crate::server::backups::backup_type::BackupType;
use crate::server::server_data::ServerData;
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

        let mut backup_data = BackupData::new(self.id, backup_filename, self.backup_type.clone(), file_size, description);

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
        let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

        let source_path = match self.backup_type {
            BackupType::World => {
                // For world backups, only backup the world directory
                let properties = self.get_server_properties().map_err(|e| anyhow::anyhow!("Failed to read server properties: {}", e))?;
                let level_name = properties.level_name.unwrap_or_else(|| "world".to_string());
                self.get_directory_path().join(level_name)
            }
            BackupType::Full => {
                // For full backups, backup the entire server directory
                self.get_directory_path()
            }
            BackupType::Incremental => {
                // For incremental backups, backup only files changed since last backup
                self.get_directory_path()
            }
        };

        if !source_path.exists() {
            return Err(anyhow::anyhow!("Source path does not exist: {:?}", source_path));
        }

        // Get last backup timestamp for incremental backups
        let last_backup_time =
            if matches!(self.backup_type, BackupType::Incremental) { self.get_last_backup_timestamp().await.unwrap_or(0) } else { 0 };

        // For incremental backups, we need to track which directories contain changed files
        let mut included_directories = std::collections::HashSet::new();

        // First pass: collect all files that need to be included
        let mut files_to_include = Vec::new();

        for entry in WalkDir::new(&source_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(&source_path)?;

            // Skip empty directory names and backup files
            if name.as_os_str().is_empty() || path.extension().is_some_and(|ext| ext == "zip" || ext == "backup") {
                continue;
            }

            if path.is_file() {
                // For incremental backups, only include files modified after last backup
                if matches!(self.backup_type, BackupType::Incremental) && last_backup_time > 0 {
                    let metadata = std::fs::metadata(path)?;
                    if let Ok(modified) = metadata.modified() {
                        let modified_timestamp = modified.duration_since(std::time::UNIX_EPOCH)?.as_secs();
                        if modified_timestamp <= last_backup_time {
                            continue; // Skip file - hasn't been modified since last backup
                        }
                    }
                }

                files_to_include.push((path.to_path_buf(), name.to_path_buf()));

                // For incremental backups, mark all parent directories as needed
                if matches!(self.backup_type, BackupType::Incremental) {
                    let mut parent_path = name.parent();
                    while let Some(parent) = parent_path {
                        included_directories.insert(parent.to_path_buf());
                        parent_path = parent.parent();
                    }
                }
            }
        }

        // Second pass: add directories (only for non-incremental backups or needed directories)
        if !matches!(self.backup_type, BackupType::Incremental) {
            // For full and world backups, include all directories
            for entry in WalkDir::new(&source_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path.strip_prefix(&source_path)?;

                if name.as_os_str().is_empty() {
                    continue;
                }

                if path.is_dir() {
                    zip.add_directory(name.to_string_lossy().replace('\\', "/"), options)?;
                }
            }
        } else {
            // For incremental backups, only add directories that contain changed files
            let mut sorted_dirs: Vec<_> = included_directories.into_iter().collect();
            sorted_dirs.sort();

            for dir_path in sorted_dirs {
                zip.add_directory(dir_path.to_string_lossy().replace('\\', "/"), options)?;
            }
        }

        // Third pass: add all the collected files
        for (file_path, name) in files_to_include {
            zip.start_file(name.to_string_lossy().replace('\\', "/"), options)?;
            let file_content = std::fs::read(file_path)?;
            std::io::copy(&mut file_content.as_slice(), &mut zip)?;
        }

        zip.finish()?;

        // Get file size
        let metadata = std::fs::metadata(backup_path)?;
        Ok(metadata.len() as i64)
    }

    async fn get_last_backup_timestamp(&self) -> Result<u64> {
        let pool = app_db::open_pool().await?;
        let backups = BackupData::list_by_server(self.id as i64, &pool).await?;
        pool.close().await;

        // Get the most recent backup timestamp
        if let Some(latest_backup) = backups.first() {
            Ok(latest_backup.created_at as u64)
        } else {
            Ok(0) // No previous backups
        }
    }

    async fn apply_backup_retention(&self, pool: &sqlx::SqlitePool) -> Result<()> {
        let backups = BackupData::list_by_server(self.id as i64, pool).await?;

        if backups.len() > self.backup_retention as usize {
            let backups_to_delete = &backups[self.backup_retention as usize..];

            for backup in backups_to_delete {
                // Delete the backup file
                let backup_path = backup.get_file_path(&self.name);
                if backup_path.exists()
                    && let Err(e) = fs::remove_file(&backup_path).await
                {
                    log::warn!("Failed to delete backup file {:?}: {}", backup_path, e);
                }

                // Delete from database
                if let Err(e) = backup.delete(pool).await {
                    log::warn!("Failed to delete backup metadata for {}: {}", backup.filename, e);
                }
            }

            log::info!("Applied backup retention for server {}: deleted {} old backups", self.name, backups_to_delete.len());
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

    pub async fn download_backup(&self, backup_id: i64) -> Result<PathBuf, String> {
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
        pool.close().await;

        // Verify backup belongs to this server
        if backup.server_id != self.id {
            return Err("Backup does not belong to this server".to_string());
        }

        let backup_path = backup.get_file_path(&self.name);

        if !backup_path.exists() {
            return Err("Backup file not found on disk".to_string());
        }

        Ok(backup_path)
    }

    pub async fn restore_backup(&self, backup_id: i64) -> Result<(), String> {
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
        pool.close().await;

        // Verify backup belongs to this server
        if backup.server_id != self.id {
            return Err("Backup does not belong to this server".to_string());
        }

        let backup_path = backup.get_file_path(&self.name);

        if !backup_path.exists() {
            return Err("Backup file not found on disk".to_string());
        }

        // Stop the server if it's running to prevent file conflicts
        if let Err(e) = self.ensure_server_stopped().await {
            return Err(format!("Failed to stop server before restoration: {}", e));
        }

        // Extract backup to the server directory
        let server_dir = self.get_directory_path();

        // Create a temporary restoration directory
        let temp_restore_dir = server_dir.parent().unwrap().join(format!("{}_restore_temp", self.name));

        // Handle restoration based on backup type
        match backup.backup_type {
            BackupType::Full | BackupType::World => {
                // For full and world backups, replace the entire directory
                match self.extract_backup(&backup_path, &temp_restore_dir).await {
                    Ok(()) => {
                        // Backup current server files (optional safety measure)
                        let current_backup_dir = server_dir.parent().unwrap().join(format!("{}_pre_restore_backup", self.name));

                        // Use robust file operations for Windows compatibility
                        if server_dir.exists()
                            && let Err(e) = self.move_directory_with_retry(&server_dir, &current_backup_dir, 3).await
                        {
                            // Clean up temp directory
                            let _ = fs::remove_dir_all(&temp_restore_dir).await;
                            return Err(format!(
                                "Failed to backup current server files: {}. This may be due to files being in use. Please ensure the server is completely stopped and no files are open in the server directory.",
                                e
                            ));
                        }

                        // Move extracted files to server directory
                        if let Err(e) = self.move_directory_with_retry(&temp_restore_dir, &server_dir, 3).await {
                            // Try to restore the original directory if the move failed
                            if current_backup_dir.exists() {
                                let _ = self.move_directory_with_retry(&current_backup_dir, &server_dir, 1).await;
                            }
                            return Err(format!("Failed to restore backup files: {}", e));
                        }

                        // Clean up the pre-restore backup
                        if current_backup_dir.exists() {
                            let _ = fs::remove_dir_all(&current_backup_dir).await;
                        }

                        log::info!("Successfully restored server '{}' from backup: {}", self.name, backup.filename);
                        Ok(())
                    }
                    Err(e) => {
                        // Clean up temp directory
                        let _ = fs::remove_dir_all(&temp_restore_dir).await;
                        Err(format!("Failed to extract backup: {}", e))
                    }
                }
            }
            BackupType::Incremental => {
                // For incremental backups, only overwrite/add files from the backup
                match self.extract_backup_selective(&backup_path, &server_dir).await {
                    Ok(()) => {
                        log::info!("Successfully restored server '{}' from incremental backup: {}", self.name, backup.filename);
                        Ok(())
                    }
                    Err(e) => {
                        Err(format!("Failed to restore incremental backup: {}", e))
                    }
                }
            }
        }
    }

    async fn ensure_server_stopped(&self) -> Result<(), String> {
        use crate::server::server_status::ServerStatus;
        use std::time::Duration;

        // Check if server is running
        let is_running = matches!(self.status, ServerStatus::Running | ServerStatus::Starting);

        if is_running {
            log::info!("Stopping server '{}' before backup restoration", self.name);

            // Get a mutable copy to stop the server
            let mut server_copy = match ServerData::get(self.id, self.owner_id).await {
                Ok(Some(server)) => server,
                Ok(None) => return Err("Server not found".to_string()),
                Err(e) => return Err(format!("Failed to get server for stopping: {}", e)),
            };

            // Stop the server
            if let Err(e) = server_copy.stop_server().await {
                return Err(format!("Failed to stop server: {}", e));
            }

            // Wait for server to stop with timeout
            let mut attempts = 0;
            let max_attempts = 30; // 30 seconds timeout

            while attempts < max_attempts {
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Refresh server status
                if let Ok(Some(updated_server)) = ServerData::get(self.id, self.owner_id).await
                    && matches!(updated_server.status, ServerStatus::Stopped | ServerStatus::Crashed)
                {
                    log::info!("Server '{}' stopped successfully", self.name);
                    return Ok(());
                }

                attempts += 1;
            }

            return Err("Server did not stop within 30 seconds. Please manually stop the server and try again.".to_string());
        }

        Ok(())
    }

    async fn move_directory_with_retry(&self, from: &Path, to: &Path, max_retries: u32) -> Result<(), String> {
        use std::time::Duration;

        for attempt in 0..max_retries {
            // First try a simple rename (fastest if it works)
            match fs::rename(from, to).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    log::warn!("Attempt {} failed to rename directory from {:?} to {:?}: {}", attempt + 1, from, to, e);

                    if attempt == max_retries - 1 {
                        // On final attempt, try copy + remove approach
                        log::info!("Final attempt using copy + remove approach");
                        return self.copy_and_remove_directory(from, to).await;
                    } else {
                        // Wait before retry with exponential backoff
                        let delay = Duration::from_millis(500 * (1 << attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err("All retry attempts failed".to_string())
    }

    async fn copy_and_remove_directory(&self, from: &Path, to: &Path) -> Result<(), String> {
        use tokio::fs;

        // Create destination directory
        fs::create_dir_all(to).await.map_err(|e| format!("Failed to create destination directory: {}", e))?;

        // Copy all contents recursively
        self.copy_dir_recursive(from, to).await?;

        // Remove source directory
        fs::remove_dir_all(from).await.map_err(|e| format!("Failed to remove source directory after copy: {}", e))?;

        Ok(())
    }

    fn copy_dir_recursive<'a>(
        &'a self,
        from: &'a Path,
        to: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
        Box::pin(async move {
            use tokio::fs;

            let mut entries = fs::read_dir(from).await.map_err(|e| format!("Failed to read source directory: {}", e))?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| format!("Failed to read directory entry: {}", e))? {
                let entry_path = entry.path();
                let file_name = entry.file_name();
                let dest_path = to.join(file_name);

                if entry_path.is_dir() {
                    // Recursively copy subdirectory
                    fs::create_dir_all(&dest_path).await.map_err(|e| format!("Failed to create subdirectory: {}", e))?;
                    self.copy_dir_recursive(&entry_path, &dest_path).await?;
                } else {
                    // Copy file
                    fs::copy(&entry_path, &dest_path).await.map_err(|e| format!("Failed to copy file {:?}: {}", entry_path, e))?;
                }
            }

            Ok(())
        })
    }

    async fn extract_backup(&self, backup_path: &Path, extract_to: &Path) -> Result<(), String> {
        // Create an extraction directory
        if let Err(e) = fs::create_dir_all(extract_to).await {
            return Err(format!("Failed to create extraction directory: {}", e));
        }

        let file = File::open(backup_path).map_err(|e| format!("Failed to open backup file: {}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read archive entry: {}", e))?;
            let outpath = extract_to.join(file.name());

            if file.name().ends_with('/') {
                // Directory entry
                std::fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                // File entry
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }

                let mut outfile = File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }

        Ok(())
    }

    async fn extract_backup_selective(&self, backup_path: &Path, server_dir: &Path) -> Result<(), String> {
        // For incremental backups, extract files directly to the server directory
        // This will only overwrite/add files that are in the backup, leaving other files untouched
        
        let file = File::open(backup_path).map_err(|e| format!("Failed to open backup file: {}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;

        log::info!("Extracting incremental backup with {} entries to server directory", archive.len());

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read archive entry: {}", e))?;
            let relative_path = file.name();
            let outpath = server_dir.join(relative_path);

            if file.name().ends_with('/') {
                // Directory entry - create if it doesn't exist
                if let Err(e) = fs::create_dir_all(&outpath).await {
                    log::warn!("Failed to create directory {:?}: {}", outpath, e);
                    // Don't fail completely, just log the warning and continue
                }
                log::debug!("Created directory: {:?}", outpath);
            } else {
                // File entry - ensure parent directory exists
                if let Some(parent) = outpath.parent() {
                    if let Err(e) = fs::create_dir_all(parent).await {
                        return Err(format!("Failed to create parent directory {:?}: {}", parent, e));
                    }
                }

                // Extract file, overwriting if it exists
                let mut outfile = File::create(&outpath).map_err(|e| format!("Failed to create file {:?}: {}", outpath, e))?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy file {:?}: {}", outpath, e))?;
                
                log::debug!("Extracted file: {:?}", outpath);
            }
        }

        log::info!("Successfully extracted incremental backup to server directory");
        Ok(())
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
        if backup_path.exists()
            && let Err(e) = fs::remove_file(&backup_path).await
        {
            log::warn!("Failed to delete backup file {:?}: {}", backup_path, e);
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
