use super::backup_data::{Backup, BackupType};
use crate::server::server_data::ServerData;
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use obsidian_backups::BackupManager;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Get the backup directory for a server
pub fn get_backup_dir(server: &ServerData) -> PathBuf {
    PathBuf::from("./backups").join(&server.name)
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

    BackupManager::new(backup_dir, server_dir)
}

/// Perform a full backup of the server
pub async fn perform_backup(
    server: &ServerData,
    backup_type: BackupType,
    description: Option<String>,
) -> Result<String> {
    info!(
        "Starting {} for server '{}' (ID: {})",
        match backup_type {
            BackupType::Full => "full backup",
            BackupType::Incremental => "incremental backup",
            BackupType::WorldOnly => "world backup",
        },
        server.name,
        server.id
    );

    let manager = create_backup_manager(server)?;

    // For world-only backups, we need to handle this specially
    if backup_type == BackupType::WorldOnly {
        return perform_world_only_backup(server, description).await;
    }

    // Create description with backup type prefix
    let full_description = match description {
        Some(desc) => format!("[{}] {}", backup_type_name(backup_type), desc),
        None => format!("[{}] Backup created at {}", backup_type_name(backup_type), chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")),
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
        BackupType::Full => "Full",
        BackupType::Incremental => "Incremental",
        BackupType::WorldOnly => "World",
    }
}

/// Perform a world-only backup
async fn perform_world_only_backup(
    server: &ServerData,
    description: Option<String>,
) -> Result<String> {
    let backup_dir = get_backup_dir(server).join("world-backups");
    let server_dir = get_server_dir(server);

    // Ensure backup directory exists
    fs::create_dir_all(&backup_dir).await?;

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

    // Create a temporary directory for world-only backup
    let temp_world_dir = backup_dir.join("temp_world");
    if temp_world_dir.exists() {
        fs::remove_dir_all(&temp_world_dir).await?;
    }
    fs::create_dir_all(&temp_world_dir).await?;

    // Copy world folders to temp directory
    for world_folder in &world_folders {
        let source = server_dir.join(world_folder);
        let dest = temp_world_dir.join(world_folder);
        copy_dir_recursive(&source, &dest).await?;
    }

    // Create description with backup type prefix
    let full_description = match description {
        Some(desc) => format!("[World] {}", desc),
        None => format!("[World] Backup created at {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")),
    };

    // Create BackupManager for the temp world directory
    let manager = BackupManager::new(&backup_dir, &temp_world_dir)?;
    let commit_id = manager
        .backup(Some(full_description))
        .map_err(|e| anyhow!("Failed to create world backup: {}", e))?;

    // Clean up temp directory
    fs::remove_dir_all(&temp_world_dir).await?;

    Ok(commit_id)
}

/// List all backups for a server
pub async fn list_backups(server: &ServerData) -> Result<Vec<Backup>> {
    let backup_dir = get_backup_dir(server);

    // Check if backup directory exists
    if !backup_dir.exists() {
        return Ok(vec![]);
    }

    let server_dir = get_server_dir(server);
    let manager = BackupManager::new(&backup_dir, &server_dir)?;

    let backup_items = manager.list().map_err(|e| anyhow!("Failed to list backups: {}", e))?;

    // Get backup directory size once
    let total_size = get_directory_size(&backup_dir).await.unwrap_or(0);

    // Convert BackupItem to our Backup struct
    let mut backups = Vec::new();
    for item in backup_items {
        // Parse backup type from description
        let backup_type = if item.description.starts_with("[Full]") {
            BackupType::Full
        } else if item.description.starts_with("[Incremental]") {
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

    Ok(backups)
}

/// Restore a backup
pub async fn restore_backup(
    server: &ServerData,
    commit_id: &str,
) -> Result<()> {
    info!(
        "Restoring backup {} for server '{}' (ID: {})",
        commit_id, server.name, server.id
    );

    // Try regular backup manager first
    let manager = create_backup_manager(server)?;

    match manager.restore(commit_id) {
        Ok(_) => {
            info!(
                "Backup {} restored successfully for server '{}'",
                commit_id, server.name
            );
            Ok(())
        }
        Err(e) => {
            // If regular restore fails, try world-only
            warn!("Regular restore failed, trying world-only: {}", e);
            let backup_dir = get_backup_dir(server).join("world-backups");
            let server_dir = get_server_dir(server);
            let world_manager = BackupManager::new(backup_dir, server_dir)?;
            world_manager.restore(commit_id).map_err(|e2| anyhow!("Failed to restore backup: {} (world-only also failed: {})", e, e2))?;
            info!(
                "Backup {} restored successfully for server '{}' (world-only)",
                commit_id, server.name
            );
            Ok(())
        }
    }
}


pub async fn delete_backup(server: &ServerData, commit_id: &str) -> Result<()> {
    info!("Deleting backup {} for server '{}'", commit_id, server.name);
    let manager = create_backup_manager(server)?;
    if let Err(e)= manager.purge_commit(commit_id) {
        return Err(anyhow!("Failed to delete backup: {}", e));
    }
    info!("Backup {} deleted successfully for server '{}'", commit_id, server.name);
    Ok(())
}

/// Export a backup as a .7z archive
pub async fn export_backup(
    server: &ServerData,
    commit_id: &str,
    output_path: &Path,
) -> Result<()> {
    info!(
        "Exporting backup {} for server '{}' to {:?}",
        commit_id, server.name, output_path
    );

    // Try regular backup manager first
    let manager = create_backup_manager(server)?;
    
    match manager.export(commit_id, output_path, 5) {
        Ok(_) => {
            info!(
                "Backup {} exported successfully to {:?}",
                commit_id, output_path
            );
            Ok(())
        }
        Err(e) => {
            // If regular export fails, try world-only
            warn!("Regular export failed, trying world-only: {}", e);
            let backup_dir = get_backup_dir(server).join("world-backups");
            let server_dir = get_server_dir(server);
            let world_manager = BackupManager::new(backup_dir, server_dir)?;
            world_manager.export(commit_id, output_path, 5).map_err(|e2| anyhow!("Failed to export backup: {} (world-only also failed: {})", e, e2))?;
            info!(
                "Backup {} exported successfully to {:?} (world-only)",
                commit_id, output_path
            );
            Ok(())
        }
    }
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

/// Copy a directory recursively
fn copy_dir_recursive<'a>(src: &'a Path, dst: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        fs::create_dir_all(dst).await?;

        let mut entries = fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if entry.metadata().await?.is_dir() {
                copy_dir_recursive(&src_path, &dst_path).await?;
            } else {
                fs::copy(&src_path, &dst_path).await?;
            }
        }

        Ok(())
    })
}
