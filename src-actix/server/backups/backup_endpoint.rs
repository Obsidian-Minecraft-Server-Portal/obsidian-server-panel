use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::UserRequestExt;
use crate::server::backups::backup_data::BackupData;
use crate::server::backups::backup_scheduler;
use crate::server::server_data::ServerData;
use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, delete, get, post, put, web};
use anyhow::Context;
use obsidian_backups::BackupManager;
use serde::{Deserialize, Serialize};
use serde_hash::hashids::decode_single;
use serde_json::json;
use std::path::PathBuf;

#[derive(Deserialize)]
struct CreateBackupRequest {
    description: Option<String>,
}

#[derive(Deserialize)]
struct UpdateBackupSettingsRequest {
    backup_enabled: Option<bool>,
    backup_cron: Option<String>,
    backup_retention: Option<u32>,
}

#[derive(Serialize)]
struct BackupListResponse {
    backup: BackupData,
    file_size_formatted: String,
    created_at_formatted: String,
}

impl BackupListResponse {
    fn from_backup_data(backup: BackupData) -> Self {
        let file_size_formatted = backup.format_file_size();
        let created_at_formatted = backup.format_created_at();
        Self {
            backup,
            file_size_formatted,
            created_at_formatted,
        }
    }
}

#[derive(Serialize)]
struct BackupSettingsResponse {
    backup_enabled: bool,
    backup_cron: String,
    backup_retention: u32,
    is_scheduled: bool,
}

#[get("")]
async fn list_backups(path: web::Path<String>, req: HttpRequest) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;

    let user = req.get_user().map_err(|_| {
        log::error!("User not found in request extensions");
        HttpResponse::Unauthorized().json(json!({
            "error": "Authentication required"
        }))
    })?;

    let user_id = user.id.ok_or_else(|| {
        log::error!("User ID not found in user data");
        HttpResponse::Unauthorized().json(json!({
            "error": "Invalid user data"
        }))
    })?;

    // Verify user has access to this server
    match ServerData::get(server_id, user_id).await {
        Ok(Some(_)) => {
            // Query backups from database
            let pool = crate::app_db::open_pool().await.map_err(|e| {
                log::error!("Failed to open database pool: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error"
                }))
            })?;
            let backups: Vec<BackupData> = sqlx::query_as(
                r#"
                SELECT id, server_id, filename, backup_type, file_size, created_at, description, git_commit_id
                FROM backups
                WHERE server_id = ?
                ORDER BY created_at DESC
                "#
            )
            .bind(server_id as i64)
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to list backups for server {}: {}", server_id, e);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to list backups"
                }))
            })?;

            let backup_responses: Vec<BackupListResponse> = backups
                .into_iter()
                .map(BackupListResponse::from_backup_data)
                .collect();

            Ok(HttpResponse::Ok().json(json!({
                "backups": backup_responses
            })))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found"
        }))),
        Err(e) => {
            log::error!("Failed to get server {}: {}", server_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get server"
            })))
        }
    }
}

#[post("")]
async fn create_backup(
    path: web::Path<String>,
    req_body: web::Json<CreateBackupRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;

    let user = req.get_user().map_err(|_| {
        log::error!("User not found in request extensions");
        HttpResponse::Unauthorized().json(json!({
            "error": "Authentication required"
        }))
    })?;

    let user_id = user.id.ok_or_else(|| {
        log::error!("User ID not found in user data");
        HttpResponse::Unauthorized().json(json!({
            "error": "Invalid user data"
        }))
    })?;

    match ServerData::get(server_id, user_id).await {
        Ok(Some(server)) => {
            // All backups are incremental only
            let result = create_incremental_backup(&server, req_body.description.clone()).await;

            match result {
                Ok(()) => Ok(HttpResponse::Ok().json(json!({
                    "message": "Backup created successfully"
                }))),
                Err(e) => {
                    log::error!("Failed to create backup for server {}: {}", server_id, e);
                    Ok(HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to create backup: {}", e)
                    })))
                }
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found"
        }))),
        Err(e) => {
            log::error!("Failed to get server {}: {}", server_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get server"
            })))
        }
    }
}

#[get("/{backup_id}/download")]
async fn download_backup(
    path: web::Path<(String, String)>,
    req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> actix_web::Result<NamedFile> {
    let (server_id, backup_id) = path.into_inner();
    let server_id: u64 = decode_single(server_id).map_err(actix_web::error::ErrorBadRequest)?;
    let backup_id: u64 = decode_single(backup_id).map_err(actix_web::error::ErrorBadRequest)?;

    let user = req
        .get_user()
        .map_err(|_| actix_web::error::ErrorUnauthorized("Authentication required"))?;

    let user_id = user
        .id
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid user data"))?;

    // Get server and verify access
    let server = match ServerData::get(server_id, user_id).await {
        Ok(Some(server)) => server,
        Ok(None) => return Err(actix_web::error::ErrorNotFound("Server not found")),
        Err(e) => {
            log::error!("Failed to get server {}: {}", server_id, e);
            return Err(actix_web::error::ErrorInternalServerError("Failed to get server"));
        }
    };

    // Get backup from database
    let pool = crate::app_db::open_pool().await.map_err(|e| {
        log::error!("Failed to open database pool: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;
    let backup: BackupData = sqlx::query_as(
        r#"
        SELECT id, server_id, filename, backup_type, file_size, created_at, description, git_commit_id
        FROM backups
        WHERE id = ? AND server_id = ?
        "#,
    )
    .bind(backup_id as i64)
    .bind(server_id as i64)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        log::error!("Failed to get backup {}: {}", backup_id, e);
        actix_web::error::ErrorNotFound("Backup not found")
    })?;

    // Check if it's a Git-based backup (all new backups are git-based)
    if backup.git_commit_id.is_some() {
        let download_type = query.get("type").map(|s| s.as_str()).unwrap_or("full");
        
        let server_path = server.get_directory_path();
        let store_path = PathBuf::from("./backup_stores").join(format!("{}", server_id));
        
        // Initialize backup manager
        let manager = BackupManager::new(&store_path, &server_path)
            .map_err(|e| {
                log::error!("Failed to initialize BackupManager: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to initialize backup manager")
            })?;
        
        let commit_id = backup.git_commit_id.as_ref().unwrap();
        
        // Create export path
        let export_filename = match download_type {
            "diff" => format!("backup_{}_diff.7z", commit_id),
            _ => format!("backup_{}_full.7z", commit_id),
        };
        
        let export_path = PathBuf::from("./temp_exports").join(&export_filename);
        
        // Create temp directory if it doesn't exist
        if let Some(parent) = export_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                log::error!("Failed to create temp export directory: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to create export directory")
            })?;
        }
        
        // Export backup
        manager.export(commit_id, &export_path, 5).map_err(|e| {
            log::error!("Failed to export backup {}: {}", commit_id, e);
            actix_web::error::ErrorInternalServerError("Failed to export backup")
        })?;
        
        // Return the exported file
        NamedFile::open(&export_path)
            .map_err(|e| {
                log::error!("Failed to open exported backup file: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to open backup file")
            })
    } else {
        // For ZIP-based backups (World or legacy Full backups)
        let backup_file_path = PathBuf::from("./backups")
            .join(server.name.clone())
            .join(&backup.filename);
        
        NamedFile::open(&backup_file_path).map_err(|e| {
            log::error!("Failed to open backup file: {}", e);
            actix_web::error::ErrorNotFound("Backup file not found")
        })
    }
}

#[post("/{backup_id}/restore")]
async fn restore_backup(
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let (server_id, backup_id) = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;
    let backup_id: u64 = decode_single(backup_id)?;

    let user = req.get_user().map_err(|_| {
        log::error!("User not found in request extensions");
        HttpResponse::Unauthorized().json(json!({
            "error": "Authentication required"
        }))
    })?;

    let user_id = user.id.ok_or_else(|| {
        log::error!("User ID not found in user data");
        HttpResponse::Unauthorized().json(json!({
            "error": "Invalid user data"
        }))
    })?;

    match ServerData::get(server_id, user_id).await {
        Ok(Some(server)) => {
            // Get backup from database
            let pool = crate::app_db::open_pool().await.map_err(|e| {
                log::error!("Failed to open database pool: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error"
                }))
            })?;
            let backup: BackupData = sqlx::query_as(
                r#"
                SELECT id, server_id, filename, backup_type, file_size, created_at, description, git_commit_id
                FROM backups
                WHERE id = ? AND server_id = ?
                "#,
            )
            .bind(backup_id as i64)
            .bind(server_id as i64)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to get backup {}: {}", backup_id, e);
                HttpResponse::NotFound().json(json!({
                    "error": "Backup not found"
                }))
            })?;

            // Restore backup (all new backups are git-based)
            let result = if backup.git_commit_id.is_some() {
                // Git-based restore
                let server_path = server.get_directory_path();
                let store_path = PathBuf::from("./backup_stores").join(format!("{}", server_id));

                let manager = BackupManager::new(&store_path, &server_path)
                    .context("Failed to initialize BackupManager");

                match manager {
                    Ok(mgr) => {
                        let commit_id = backup.git_commit_id.as_ref().unwrap();
                        mgr.restore(commit_id)
                            .context("Failed to restore backup")
                    }
                    Err(e) => Err(e),
                }
            } else {
                // Legacy ZIP-based restore (for old backups)
                Err(anyhow::anyhow!("ZIP-based backup restore not yet implemented"))
            };

            match result {
                Ok(()) => Ok(HttpResponse::Ok().json(json!({
                    "message": "Backup restored successfully"
                }))),
                Err(e) => {
                    log::error!(
                        "Failed to restore backup {} for server {}: {}",
                        backup_id,
                        server_id,
                        e
                    );
                    Ok(HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to restore backup: {}", e)
                    })))
                }
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found"
        }))),
        Err(e) => {
            log::error!("Failed to get server {}: {}", server_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get server"
            })))
        }
    }
}

#[delete("/{backup_id}")]
async fn delete_backup(
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let (server_id, backup_id) = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;
    let backup_id: u64 = decode_single(backup_id)?;

    let user = req.get_user().map_err(|_| {
        log::error!("User not found in request extensions");
        HttpResponse::Unauthorized().json(json!({
            "error": "Authentication required"
        }))
    })?;

    let user_id = user.id.ok_or_else(|| {
        log::error!("User ID not found in user data");
        HttpResponse::Unauthorized().json(json!({
            "error": "Invalid user data"
        }))
    })?;

    match ServerData::get(server_id, user_id).await {
        Ok(Some(server)) => {
            // Get backup from database
            let pool = crate::app_db::open_pool().await.map_err(|e| {
                log::error!("Failed to open database pool: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error"
                }))
            })?;
            let backup: BackupData = sqlx::query_as(
                r#"
                SELECT id, server_id, filename, backup_type, file_size, created_at, description, git_commit_id
                FROM backups
                WHERE id = ? AND server_id = ?
                "#,
            )
            .bind(backup_id as i64)
            .bind(server_id as i64)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to get backup {}: {}", backup_id, e);
                HttpResponse::NotFound().json(json!({
                    "error": "Backup not found"
                }))
            })?;

            // Delete backup file if it's a legacy ZIP-based backup
            if backup.git_commit_id.is_none() {
                let backup_file_path = PathBuf::from("./backups")
                    .join(server.name.clone())
                    .join(&backup.filename);

                if backup_file_path.exists() {
                    if let Err(e) = std::fs::remove_file(&backup_file_path) {
                        log::warn!(
                            "Failed to delete backup file {}: {}",
                            backup_file_path.display(),
                            e
                        );
                    }
                }
            }
            // Note: Git-based backups are managed by the git repository, no file deletion needed

            // Delete from database
            sqlx::query("DELETE FROM backups WHERE id = ?")
                .bind(backup_id as i64)
                .execute(&pool)
                .await
                .map_err(|e| {
                    log::error!("Failed to delete backup {} from database: {}", backup_id, e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to delete backup"
                    }))
                })?;

            Ok(HttpResponse::Ok().json(json!({
                "message": "Backup deleted successfully"
            })))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found"
        }))),
        Err(e) => {
            log::error!("Failed to get server {}: {}", server_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get server"
            })))
        }
    }
}

#[get("/settings")]
async fn get_backup_settings(path: web::Path<String>, req: HttpRequest) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;

    let user = req.get_user().map_err(|_| {
        log::error!("User not found in request extensions");
        HttpResponse::Unauthorized().json(json!({
            "error": "Authentication required"
        }))
    })?;

    let user_id = user.id.ok_or_else(|| {
        log::error!("User ID not found in user data");
        HttpResponse::Unauthorized().json(json!({
            "error": "Invalid user data"
        }))
    })?;

    match ServerData::get(server_id, user_id).await {
        Ok(Some(server)) => {
            let is_scheduled = backup_scheduler::is_server_scheduled(server_id).await;

            let settings = BackupSettingsResponse {
                backup_enabled: server.backup_enabled,
                backup_cron: server.backup_cron,
                backup_retention: server.backup_retention,
                is_scheduled,
            };

            Ok(HttpResponse::Ok().json(settings))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found"
        }))),
        Err(e) => {
            log::error!("Failed to get server {}: {}", server_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get server"
            })))
        }
    }
}

#[put("/settings")]
async fn update_backup_settings(
    path: web::Path<String>,
    req_body: web::Json<UpdateBackupSettingsRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;

    let user = req.get_user().map_err(|_| {
        log::error!("User not found in request extensions");
        HttpResponse::Unauthorized().json(json!({
            "error": "Authentication required"
        }))
    })?;

    let user_id = user.id.ok_or_else(|| {
        log::error!("User ID not found in user data");
        HttpResponse::Unauthorized().json(json!({
            "error": "Invalid user data"
        }))
    })?;

    match ServerData::get(server_id, user_id).await {
        Ok(Some(mut server)) => {
            // Update server settings (all backups are incremental only)
            if let Some(backup_enabled) = req_body.backup_enabled {
                server.backup_enabled = backup_enabled;
            }
            if let Some(backup_cron) = &req_body.backup_cron {
                server.backup_cron = backup_cron.clone();
            }
            if let Some(backup_retention) = req_body.backup_retention {
                server.backup_retention = backup_retention;
            }

            // Save to database
            match server.save().await {
                Ok(()) => {
                    // Update scheduler
                    if server.backup_enabled {
                        if let Err(e) = backup_scheduler::reschedule_server_backup(&server).await {
                            log::error!(
                                "Failed to reschedule backup for server {}: {}",
                                server_id,
                                e
                            );
                        }
                    } else if let Err(e) = backup_scheduler::unschedule_server_backup(server_id).await
                    {
                        log::error!(
                            "Failed to unschedule backup for server {}: {}",
                            server_id,
                            e
                        );
                    }

                    Ok(HttpResponse::Ok().json(json!({
                        "message": "Backup settings updated successfully"
                    })))
                }
                Err(e) => {
                    log::error!("Failed to save server {}: {}", server_id, e);
                    Ok(HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to save backup settings"
                    })))
                }
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found"
        }))),
        Err(e) => {
            log::error!("Failed to get server {}: {}", server_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get server"
            })))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/backups")
            .service(list_backups)
            .service(create_backup)
            .service(download_backup)
            .service(restore_backup)
            .service(delete_backup)
            .service(get_backup_settings)
            .service(update_backup_settings)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found"
                }))
            })),
    );
}

// Helper function to create incremental backup
async fn create_incremental_backup(
    server: &ServerData,
    description: Option<String>,
) -> anyhow::Result<()> {
    let server_path = server.get_directory_path();
    let store_path = PathBuf::from("./backup_stores").join(format!("{}", server.id));

    // Initialize backup manager
    let manager =
        BackupManager::new(&store_path, &server_path).context("Failed to initialize BackupManager")?;

    // Create backup
    let backup_id = manager
        .backup(description.clone())
        .context("Failed to create backup")?;

    log::info!(
        "Created incremental backup {} for server {}",
        backup_id,
        server.id
    );

    // Save backup metadata to database
    let pool = crate::app_db::open_pool().await?;

    let filename = description
        .as_ref()
        .map(|d| format!("{}.git", d))
        .unwrap_or_else(|| format!("backup_{}.git", chrono::Utc::now().format("%Y%m%d_%H%M%S")));

    sqlx::query(
        r#"
        INSERT INTO backups (server_id, filename, backup_type, file_size, description, git_commit_id, created_at)
        VALUES (?, ?, 1, 0, ?, ?, STRFTIME('%s', 'now'))
        "#,
    )
    .bind(server.id as i64)
    .bind(filename)
    .bind(description)
    .bind(&backup_id)
    .execute(&pool)
    .await
    .context("Failed to save backup to database")?;

    // Apply retention policy
    apply_retention_policy(server).await?;

    Ok(())
}

// Helper function to apply retention policy
async fn apply_retention_policy(server: &ServerData) -> anyhow::Result<()> {
    let pool = crate::app_db::open_pool().await?;

    // Get all incremental backups for this server, ordered by creation date
    let backups: Vec<(i64, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, git_commit_id
        FROM backups
        WHERE server_id = ? AND backup_type = 1
        ORDER BY created_at DESC
        "#,
    )
    .bind(server.id as i64)
    .fetch_all(&pool)
    .await
    .context("Failed to fetch backups for retention policy")?;

    // Keep only the specified number of backups
    if backups.len() > server.backup_retention as usize {
        let backups_to_delete = &backups[server.backup_retention as usize..];

        for (backup_id, _) in backups_to_delete {
            // Delete from database
            sqlx::query("DELETE FROM backups WHERE id = ?")
                .bind(backup_id)
                .execute(&pool)
                .await
                .context("Failed to delete old backup from database")?;

            log::info!(
                "Deleted old backup {} for server {} (retention policy)",
                backup_id,
                server.id
            );
        }
    }

    Ok(())
}
