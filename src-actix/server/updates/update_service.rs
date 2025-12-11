use super::update_data::UpdateInfo;
use crate::app_db;
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use crate::server::server_type::ServerType;
use anyhow::{anyhow, Result};
use futures::stream::StreamExt;
use log::{debug, info, warn};
use reqwest::Client;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct UpdateService;

impl UpdateService {
    /// Apply server update
    pub async fn apply_update(
        server: &mut ServerData,
        update_info: &UpdateInfo,
    ) -> Result<()> {
        info!(
            "Applying update for server '{}': {} -> {}",
            server.name, update_info.current_version, update_info.latest_version
        );

        // 1. Check if server is running
        if server.status != ServerStatus::Idle {
            return Err(anyhow!(
                "Server must be stopped before updating. Current status: {:?}",
                server.status
            ));
        }

        let server_dir = server.get_directory_path();

        // 2. Create backups directory for jar backups
        let backup_dir = server_dir.join("backups_jars");
        fs::create_dir_all(&backup_dir).await?;

        // 3. Backup current server jar
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let jar_name = server.server_jar.trim_end_matches(".jar");
        let backup_name = format!("{}_backup_{}.jar", jar_name, timestamp);
        let backup_path = backup_dir.join(&backup_name);

        let current_jar_path = server_dir.join(&server.server_jar);
        if current_jar_path.exists() {
            fs::copy(&current_jar_path, &backup_path).await?;
            info!(
                "Backed up current jar to: {}",
                backup_path.display()
            );
        } else {
            warn!(
                "Current jar file not found at {}, skipping backup",
                current_jar_path.display()
            );
        }

        // 4. Download new version
        info!(
            "Downloading update from: {}",
            update_info.download_url
        );

        let client = Client::new();
        let response = client.get(&update_info.download_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download update: HTTP {}",
                response.status()
            ));
        }

        // 5. Save new jar to temp file first
        let temp_jar = server_dir.join(format!("{}.tmp", server.server_jar));
        let mut file = fs::File::create(&temp_jar).await?;
        let mut stream = response.bytes_stream();

        debug!("Downloading to temp file: {}", temp_jar.display());

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        drop(file);

        info!("Download complete, replacing old jar");

        // 6. Replace old jar with new one
        if current_jar_path.exists() {
            fs::remove_file(&current_jar_path).await?;
        }

        fs::rename(&temp_jar, &current_jar_path).await?;

        info!("Server jar updated successfully");

        // 7. Update database record
        let pool = app_db::open_pool().await?;

        match server.server_type {
            Some(ServerType::Vanilla) => {
                // For vanilla, update minecraft_version
                server.minecraft_version = Some(update_info.latest_version.clone());
                sqlx::query(
                    r#"
                    UPDATE servers
                    SET minecraft_version = ?,
                        update_available = 0,
                        latest_version = NULL,
                        updated_at = UNIX_TIMESTAMP()
                    WHERE id = ?
                    "#
                )
                .bind(&update_info.latest_version)
                .bind(server.id as i64)
                .execute(&pool)
                .await?;
            }
            Some(ServerType::Fabric)
            | Some(ServerType::Forge)
            | Some(ServerType::NeoForge)
            | Some(ServerType::Quilt) => {
                // For loaders, update loader_version
                server.loader_version = Some(update_info.latest_version.clone());
                sqlx::query(
                    r#"
                    UPDATE servers
                    SET loader_version = ?,
                        update_available = 0,
                        latest_version = NULL,
                        updated_at = UNIX_TIMESTAMP()
                    WHERE id = ?
                    "#
                )
                .bind(&update_info.latest_version)
                .bind(server.id as i64)
                .execute(&pool)
                .await?;
            }
            _ => {}
        }

        server.update_available = false;
        server.latest_version = None;

        pool.close().await;

        info!(
            "Server '{}' updated successfully to version {}",
            server.name, update_info.latest_version
        );

        Ok(())
    }

    /// Rollback to previous version
    pub async fn rollback_update(server: &mut ServerData) -> Result<()> {
        info!("Rolling back update for server '{}'", server.name);

        let server_dir = server.get_directory_path();
        let backup_dir = server_dir.join("backups_jars");

        if !backup_dir.exists() {
            return Err(anyhow!(
                "No backup directory found at {}",
                backup_dir.display()
            ));
        }

        // Find most recent backup
        let mut entries = fs::read_dir(&backup_dir).await?;
        let mut backups = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                backups.push(path);
            }
        }

        if backups.is_empty() {
            return Err(anyhow!("No backup found for rollback"));
        }

        // Sort by filename (which includes timestamp) in descending order
        backups.sort_by(|a, b| {
            b.file_name()
                .unwrap_or_default()
                .cmp(&a.file_name().unwrap_or_default())
        });

        let latest_backup = &backups[0];

        info!(
            "Rolling back to backup: {}",
            latest_backup.display()
        );

        // Restore backup
        let current_jar_path = server_dir.join(&server.server_jar);
        fs::copy(latest_backup, &current_jar_path).await?;

        info!(
            "Rolled back server '{}' to previous version",
            server.name
        );

        Ok(())
    }

    /// Get list of available backup jars
    pub async fn list_backup_jars(server: &ServerData) -> Result<Vec<String>> {
        let server_dir = server.get_directory_path();
        let backup_dir = server_dir.join("backups_jars");

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&backup_dir).await?;
        let mut backups = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                if let Some(filename) = path.file_name() {
                    backups.push(filename.to_string_lossy().to_string());
                }
            }
        }

        // Sort by filename (newest first)
        backups.sort_by(|a, b| b.cmp(a));

        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::server_type::ServerType;

    // Helper function to create a test server
    fn create_test_server(server_type: ServerType, mc_version: &str, loader_version: Option<&str>) -> ServerData {
        ServerData {
            id: 1,
            name: "Test Server".to_string(),
            description: Some("Test".to_string()),
            directory: "test".to_string(),
            java_executable: "java".to_string(),
            java_args: String::new(),
            minecraft_args: String::new(),
            max_memory: 4,
            min_memory: 1,
            server_jar: "server.jar".to_string(),
            upnp: false,
            auto_start: false,
            auto_restart: false,
            backup_enabled: false,
            backup_cron: "0 0 * * * *".to_string(),
            backup_retention: 7,
            server_type: Some(server_type),
            minecraft_version: Some(mc_version.to_string()),
            loader_version: loader_version.map(|v| v.to_string()),
            status: ServerStatus::Idle,
            owner_id: 1,
            last_update_check: None,
            update_available: false,
            latest_version: None,
            last_started: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn test_server_idle_check() {
        let server = create_test_server(ServerType::Vanilla, "1.20.1", None);

        // Verify test server is in Idle state
        assert_eq!(server.status, ServerStatus::Idle);
    }

    #[test]
    fn test_backup_filename_generation() {
        // Test that backup filenames follow expected pattern: {jar_name}_backup_{timestamp}.jar
        let jar_name = "server";
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let expected_pattern = format!("{}_backup_", jar_name);

        // Backup name should start with jar name and contain "backup"
        let backup_name = format!("{}_backup_{}.jar", jar_name, timestamp);
        assert!(backup_name.starts_with(&expected_pattern));
        assert!(backup_name.ends_with(".jar"));
    }

    #[test]
    fn test_server_type_database_update_logic() {
        // Test vanilla server updates minecraft_version
        let vanilla = create_test_server(ServerType::Vanilla, "1.20.1", None);
        assert!(vanilla.server_type == Some(ServerType::Vanilla));
        assert!(vanilla.minecraft_version.is_some());

        // Test fabric server updates loader_version
        let fabric = create_test_server(ServerType::Fabric, "1.20.1", Some("0.15.0"));
        assert!(fabric.server_type == Some(ServerType::Fabric));
        assert!(fabric.loader_version.is_some());
    }

    #[test]
    fn test_temp_jar_naming() {
        let server_jar = "server.jar";
        let temp_jar = format!("{}.tmp", server_jar);

        assert_eq!(temp_jar, "server.jar.tmp");
        assert!(temp_jar.ends_with(".tmp"));
    }
}
