use super::backup_data::{Backup, BackupType};
use crate::server::server_data::ServerData;
use anyhow::{anyhow, Result};
use chrono::{Datelike, Timelike, Utc};
use log::{debug, info, warn};
use obsidian_backups::BackupManager;
use std::fs::File;
use std::path::{Path, PathBuf};
use tokio::fs;
use zip::{write::FileOptions, ZipWriter};

/// Get the backup directory for a server
pub fn get_backup_dir(server: &ServerData) -> PathBuf {
    let backups_dir = if let Ok(settings) = crate::settings::load_settings() {
        settings.storage.backups_directory
    } else {
        PathBuf::from("./meta/backups")
    };
    backups_dir.join(&server.name)
}

/// Get the server directory path
fn get_server_dir(server: &ServerData) -> PathBuf {
    server.get_directory_path()
}

/// Create a BackupManager instance for a server
pub fn create_backup_manager(server: &ServerData) -> Result<BackupManager> {
    let backup_dir = get_backup_dir(server);
    let server_dir = get_server_dir(server);

    // Ensure backup directory exists
    std::fs::create_dir_all(&backup_dir)?;

    debug!(
        "Creating BackupManager for server '{}': backup_dir={:?}, server_dir={:?}",
        server.name, backup_dir, server_dir
    );

    let mut manager = BackupManager::new(backup_dir, server_dir.clone())?;

    // Setup ignore patterns
    // 1. Add backups/ folder to auto-ignore (WorldEdit backups)
    let obakignore_path = server_dir.join(".obakignore");

    // Read existing .obakignore or create default
    let ignore_content = if obakignore_path.exists() {
        std::fs::read_to_string(&obakignore_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Ensure backups/ is in the ignore list
    let mut updated_ignore = ignore_content;
    if !updated_ignore.contains("backups/") {
        if !updated_ignore.is_empty() && !updated_ignore.ends_with('\n') {
            updated_ignore.push('\n');
        }
        updated_ignore.push_str("# WorldEdit backup directory (auto-added)\nbackups/\n");

        // Write back to .obakignore
        if let Err(e) = std::fs::write(&obakignore_path, &updated_ignore) {
            warn!("Failed to update .obakignore: {}", e);
        }
    }

    // Apply ignore file to BackupManager
    if obakignore_path.exists() {
        if let Err(e) = manager.setup_ignore_file(&obakignore_path) {
            warn!("Failed to setup ignore file: {}", e);
        } else {
            debug!("Ignore file configured for backup manager");
        }
    }

    Ok(manager)
}

/// Perform a backup of the server
pub async fn perform_backup(
    server: &ServerData,
    backup_type: BackupType,
    description: Option<String>,
) -> Result<String> {
    info!(
        "Starting {} for server '{}' (ID: {})",
        match backup_type {
            BackupType::Incremental => "incremental backup",
            BackupType::WorldOnly => "world backup",
        },
        server.name,
        server.id
    );

    // For world-only backups, we need to handle this specially
    if backup_type == BackupType::WorldOnly {
        return perform_world_only_backup(server, description).await;
    }

    let manager = create_backup_manager(server)?;

    // Create description with backup type prefix
    let full_description = match description {
        Some(desc) => format!("[{}] {}", backup_type_name(backup_type), desc),
        None => format!("[{}] Backup created at {}", backup_type_name(backup_type), Utc::now().format("%Y-%m-%d %H:%M:%S")),
    };

    // Perform the backup using obsidian-backups
    let commit_id = manager
        .backup(Some(full_description))
        .map_err(|e| anyhow!("Failed to create backup: {}", e))?;

    info!(
        "Backup created successfully for server '{}' with commit ID: {}",
        server.name, commit_id
    );

    Ok(commit_id)
}

/// Get the display name for backup type
fn backup_type_name(backup_type: BackupType) -> &'static str {
    match backup_type {
        BackupType::Incremental => "Incremental",
        BackupType::WorldOnly => "World",
    }
}

/// Perform a world-only backup (WorldEdit-compatible)
async fn perform_world_only_backup(
    server: &ServerData,
    _description: Option<String>,
) -> Result<String> {
    let server_dir = get_server_dir(server);
    let backups_dir = server_dir.join("backups");

    // Ensure backups directory exists
    fs::create_dir_all(&backups_dir).await?;

    // Find world folders
    let world_folders = find_world_folders(&server_dir).await?;

    if world_folders.is_empty() {
        return Err(anyhow!("No world folders found in server directory"));
    }

    info!(
        "Found {} world folders for backup: {:?}",
        world_folders.len(),
        world_folders
    );

    // Create filename using WorldEdit format: YYYY-MM-DD-HH-MM-SS.zip
    let now = Utc::now();
    let filename = format!(
        "{:04}-{:02}-{:02}-{:02}-{:02}-{:02}.zip",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    let zip_path = backups_dir.join(&filename);

    info!(
        "Creating WorldEdit backup for server '{}' at {:?}",
        server.name, zip_path
    );

    // Create ZIP file in a blocking task to avoid blocking the async runtime
    let server_dir_clone = server_dir.clone();
    let world_folders_clone = world_folders.clone();
    let zip_path_clone = zip_path.clone();

    tokio::task::spawn_blocking(move || {
        create_worldedit_zip(&server_dir_clone, &world_folders_clone, &zip_path_clone)
    })
    .await??;

    info!(
        "WorldEdit backup created successfully: {}",
        filename
    );

    // Return the filename as the backup ID
    Ok(filename)
}

/// Create a WorldEdit-compatible ZIP file
fn create_worldedit_zip(
    server_dir: &Path,
    world_folders: &[String],
    output_path: &Path,
) -> Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Add each world folder to the ZIP
    for world_folder in world_folders {
        let world_path = server_dir.join(world_folder);
        add_directory_to_zip(&mut zip, &world_path, world_folder, &options)?;
    }

    zip.finish()?;
    Ok(())
}

/// Recursively add a directory to a ZIP archive
fn add_directory_to_zip(
    zip: &mut ZipWriter<File>,
    dir_path: &Path,
    prefix: &str,
    options: &FileOptions<()>,
) -> Result<()> {
    let entries = std::fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let zip_path = format!("{}/{}", prefix, name.to_string_lossy());

        if path.is_file() {
            zip.start_file(&zip_path, *options)?;
            let mut file = File::open(&path)?;
            std::io::copy(&mut file, zip)?;
        } else if path.is_dir() {
            // Add directory entry
            zip.add_directory(&zip_path, *options)?;
            // Recursively add contents
            add_directory_to_zip(zip, &path, &zip_path, options)?;
        }
    }

    Ok(())
}

/// List all backups for a server
pub async fn list_backups(server: &ServerData) -> Result<Vec<Backup>> {
    let mut backups = Vec::new();

    // 1. List Git backups (incremental)
    let backup_dir = get_backup_dir(server);
    if backup_dir.exists() {
        let server_dir = get_server_dir(server);
        let manager = BackupManager::new(&backup_dir, &server_dir)?;

        let backup_items = manager.list().map_err(|e| anyhow!("Failed to list backups: {}", e))?;

        // Get backup directory size once
        let total_size = get_directory_size(&backup_dir).await.unwrap_or(0);

        // Convert BackupItem to our Backup struct
        for item in backup_items {
            // Parse backup type from description
            let backup_type = if item.description.starts_with("[Incremental]") {
                BackupType::Incremental
            } else if item.description.starts_with("[World]") {
                BackupType::WorldOnly
            } else {
                BackupType::Incremental // Default
            };

            backups.push(Backup {
                id: item.id,
                created_at: item.timestamp.timestamp(),
                description: item.description,
                file_size: total_size as i64 / (backups.len() as i64 + 1).max(1), // Approximate per backup
                backup_type,
            });
        }
    }

    // 2. List World-Only backups (WorldEdit ZIPs)
    let server_dir = get_server_dir(server);
    let worldedit_backups_dir = server_dir.join("backups");

    if worldedit_backups_dir.exists() {
        let mut entries = fs::read_dir(&worldedit_backups_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file()
                && let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".zip") {
                        // Parse WorldEdit format: YYYY-MM-DD-HH-MM-SS.zip
                        if let Some(timestamp) = parse_worldedit_filename(&filename_str) {
                            let metadata = entry.metadata().await?;
                            let file_size = metadata.len() as i64;

                            backups.push(Backup {
                                id: filename_str.to_string(), // Use filename as ID for World-Only backups
                                created_at: timestamp,
                                description: format!("[World] WorldEdit backup from {}", filename_str.trim_end_matches(".zip")),
                                file_size,
                                backup_type: BackupType::WorldOnly,
                            });
                        }
                    }
                }
        }
    }

    // Sort backups by timestamp (newest first)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(backups)
}

/// Parse WorldEdit filename format (YYYY-MM-DD-HH-MM-SS.zip) to Unix timestamp
fn parse_worldedit_filename(filename: &str) -> Option<i64> {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    // Remove .zip extension
    let name = filename.strip_suffix(".zip")?;

    // Split by hyphen: YYYY-MM-DD-HH-MM-SS
    let parts: Vec<&str> = name.split('-').collect();
    if parts.len() != 6 {
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    let hour: u32 = parts[3].parse().ok()?;
    let minute: u32 = parts[4].parse().ok()?;
    let second: u32 = parts[5].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)?;
    let datetime = NaiveDateTime::new(date, time);

    Some(datetime.and_utc().timestamp())
}

/// Restore a backup
pub async fn restore_backup(
    server: &ServerData,
    backup_id: &str,
) -> Result<()> {
    info!(
        "Restoring backup {} for server '{}' (ID: {})",
        backup_id, server.name, server.id
    );

    // Check if this is a World-Only backup (filename ends with .zip)
    if backup_id.ends_with(".zip") {
        return restore_worldedit_backup(server, backup_id).await;
    }

    // Regular Git backup restore
    let manager = create_backup_manager(server)?;
    manager.restore(backup_id).map_err(|e| anyhow!("Failed to restore backup: {}", e))?;

    info!(
        "Backup {} restored successfully for server '{}'",
        backup_id, server.name
    );

    Ok(())
}

/// Restore a WorldEdit backup (unzip to server directory)
async fn restore_worldedit_backup(server: &ServerData, filename: &str) -> Result<()> {
    let server_dir = get_server_dir(server);
    let backup_path = server_dir.join("backups").join(filename);

    if !backup_path.exists() {
        return Err(anyhow!("WorldEdit backup file not found: {}", filename));
    }

    info!(
        "Restoring WorldEdit backup {} for server '{}'",
        filename, server.name
    );

    // Extract ZIP in blocking task
    let server_dir_clone = server_dir.clone();
    let backup_path_clone = backup_path.clone();

    tokio::task::spawn_blocking(move || {
        extract_worldedit_backup(&backup_path_clone, &server_dir_clone)
    })
    .await??;

    info!(
        "WorldEdit backup {} restored successfully for server '{}'",
        filename, server.name
    );

    Ok(())
}

/// Extract a WorldEdit ZIP backup to the server directory
fn extract_worldedit_backup(zip_path: &Path, server_dir: &Path) -> Result<()> {
    use zip::ZipArchive;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = server_dir.join(file.name());

        if file.name().ends_with('/') {
            // Directory
            std::fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}


pub async fn delete_backup(server: &ServerData, backup_id: &str) -> Result<()> {
    info!("Deleting backup {} for server '{}'", backup_id, server.name);

    // Check if this is a World-Only backup (filename ends with .zip)
    if backup_id.ends_with(".zip") {
        let server_dir = get_server_dir(server);
        let backup_path = server_dir.join("backups").join(backup_id);

        if !backup_path.exists() {
            return Err(anyhow!("WorldEdit backup file not found: {}", backup_id));
        }

        fs::remove_file(&backup_path).await?;
        info!("WorldEdit backup {} deleted successfully for server '{}'", backup_id, server.name);
        return Ok(());
    }

    // Regular Git backup deletion
    let manager = create_backup_manager(server)?;
    if let Err(e) = manager.purge_commit(backup_id) {
        return Err(anyhow!("Failed to delete backup: {}", e));
    }
    info!("Backup {} deleted successfully for server '{}'", backup_id, server.name);
    Ok(())
}

/// Get the path to a WorldEdit backup file (for direct downloads)
pub fn get_worldedit_backup_path(server: &ServerData, filename: &str) -> Result<PathBuf> {
    let server_dir = get_server_dir(server);
    let backup_path = server_dir.join("backups").join(filename);

    if !backup_path.exists() {
        return Err(anyhow!("WorldEdit backup file not found: {}", filename));
    }

    Ok(backup_path)
}

/// Get the size of a directory recursively
fn get_directory_size<'a>(path: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + Send + 'a>> {
    Box::pin(async move {
        let mut total_size = 0u64;

        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += get_directory_size(&entry.path()).await?;
            }
        }

        Ok(total_size)
    })
}

/// Find world folders in the server directory
async fn find_world_folders(server_dir: &Path) -> Result<Vec<String>> {
    let mut world_folders = Vec::new();

    let mut entries = fs::read_dir(server_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.metadata().await?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Check if it's a world folder (contains level.dat)
            let level_dat = entry.path().join("level.dat");
            if level_dat.exists() {
                world_folders.push(name);
            }
        }
    }

    Ok(world_folders)
}

/// Detect if WorldEdit is installed on the server
pub async fn is_worldedit_installed(server: &ServerData) -> bool {
    let server_dir = get_server_dir(server);

    // Check plugins directory (Bukkit/Spigot/Paper)
    let plugins_dir = server_dir.join("plugins");
    if plugins_dir.exists()
        && let Ok(mut entries) = fs::read_dir(&plugins_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.starts_with("worldedit") && name.ends_with(".jar") {
                    return true;
                }
            }
        }

    // Check mods directory (Fabric/Forge/NeoForge/Quilt)
    let mods_dir = server_dir.join("mods");
    if mods_dir.exists()
        && let Ok(mut entries) = fs::read_dir(&mods_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.starts_with("worldedit") && name.ends_with(".jar") {
                    return true;
                }
            }
        }

    false
}

