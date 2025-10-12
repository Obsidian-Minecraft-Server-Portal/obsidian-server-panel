use super::backup_data::{BackupScheduleRequest, BackupSettings, CreateBackupRequest, IgnoreEntry, IgnoreList};
use super::{backup_db, backup_service};
use crate::actix_util::http_error::Result;
use crate::app_db;
use crate::authentication::auth_data::UserRequestExt;
use crate::server::server_data::ServerData;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use anyhow::anyhow;
use log::{error, info};
use serde_hash::hashids::decode_single;
use serde_json::json;
use tokio::fs;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/backups")
            .service(list_backups)
            .service(create_backup)
            .service(get_backup_settings)
            .service(update_backup_settings)
            .service(get_ignore_list)
            .service(update_ignore_list)
            .service(delete_schedule)
            .service(delete_backup)
            .service(restore_backup)
            .service(download_backup),
    );
}

/// GET /api/server/:id/backups - List all backups for a server
#[get("")]
async fn list_backups(
    server_id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    match backup_service::list_backups(&server).await {
        Ok(backups) => Ok(HttpResponse::Ok().json(backups)),
        Err(e) => {
            error!("Failed to list backups: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to list backups"
            })))
        }
    }
}

/// POST /api/server/:id/backups - Create a new backup
#[post("")]
async fn create_backup(
    server_id: web::Path<String>,
    request: web::Json<CreateBackupRequest>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    info!(
        "Creating backup for server '{}' (type: {:?})",
        server.name, request.backup_type
    );

    match backup_service::perform_backup(
        &server,
        request.backup_type,
        request.description.clone(),
    )
    .await
    {
        Ok(commit_id) => {
            Ok(HttpResponse::Ok().json(json!({
                "commit_id": commit_id,
                "message": "Backup created successfully"
            })))
        }
        Err(e) => {
            error!("Failed to create backup: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to create backup: {}", e)
            })))
        }
    }
}

/// DELETE /api/server/:id/backups/:backupId - Delete a backup
#[delete("/{commit_id}")]
async fn delete_backup(
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let (server_id, _commit_id) = path.into_inner();
    let server_id = decode_single(server_id)?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let _server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    // Note: obsidian-backups doesn't provide a delete method, so we return an error
    Ok(HttpResponse::BadRequest().json(json!({
        "error": "Backup deletion is not supported. Backups are managed by git and should be manually cleaned up if needed."
    })))
}

/// POST /api/server/:id/backups/:backupId/restore - Restore from a backup
#[post("/{commit_id}/restore")]
async fn restore_backup(
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let (server_id, commit_id) = path.into_inner();
    let server_id = decode_single(server_id)?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    info!(
        "Restoring backup {} for server '{}'",
        commit_id, server.name
    );

    match backup_service::restore_backup(&server, &commit_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "message": "Backup restored successfully"
        }))),
        Err(e) => {
            error!("Failed to restore backup: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to restore backup: {}", e)
            })))
        }
    }
}

/// GET /api/server/:id/backups/:backupId/download - Download a backup as .7z archive
#[get("/{commit_id}/download")]
async fn download_backup(
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let (server_id, commit_id) = path.into_inner();
    let server_id = decode_single(server_id)?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    // Create temporary file for the export
    let temp_dir = std::env::temp_dir();
    let export_filename = format!("backup_{}_{}.7z", server.name, &commit_id[..8]);
    let export_path = temp_dir.join(&export_filename);

    // Export the backup
    match backup_service::export_backup(&server, &commit_id, &export_path).await {
        Ok(_) => {
            // Read the file and send it
            match tokio::fs::read(&export_path).await {
                Ok(file_data) => {
                    // Clean up the temp file
                    let _ = tokio::fs::remove_file(&export_path).await;

                    Ok(HttpResponse::Ok()
                        .content_type("application/x-7z-compressed")
                        .insert_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", export_filename),
                        ))
                        .body(file_data))
                }
                Err(e) => {
                    error!("Failed to read exported backup file: {}", e);
                    Ok(HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to read backup file"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Failed to export backup: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to export backup: {}", e)
            })))
        }
    }
}

/// GET /api/server/:id/backups/settings - Get backup configuration
#[get("/settings")]
async fn get_backup_settings(
    server_id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let _server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    let pool = app_db::open_pool().await?;

    match backup_db::list_schedules(server_id as i64, &pool).await {
        Ok(schedules) => {
            let settings = BackupSettings { schedules };
            Ok(HttpResponse::Ok().json(settings))
        }
        Err(e) => {
            error!("Failed to get backup settings: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get backup settings"
            })))
        }
    }
}

/// PUT /api/server/:id/backups/settings - Update backup settings
#[put("/settings")]
async fn update_backup_settings(
    server_id: web::Path<String>,
    request: web::Json<BackupScheduleRequest>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let _server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    let pool = app_db::open_pool().await?;

    // Create a new schedule
    match backup_db::create_schedule(
        server_id as i64,
        request.interval_amount,
        request.interval_unit.clone(),
        request.backup_type,
        request.enabled,
        request.retention_days,
        &pool,
    )
    .await
    {
        Ok(schedule) => Ok(HttpResponse::Ok().json(schedule)),
        Err(e) => {
            error!("Failed to create backup schedule: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create backup schedule"
            })))
        }
    }
}

/// GET /api/server/:id/backups/ignore - Get the .obakignore file contents
#[get("/ignore")]
async fn get_ignore_list(
    server_id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    let ignore_path = server.get_directory_path().join(".obakignore");

    // If file doesn't exist, create it with default content
    if !ignore_path.exists() {
        let default_content = "# Backup ignore patterns\n# Patterns follow .gitignore syntax\n\n";
        if let Err(e) = fs::write(&ignore_path, default_content).await {
            error!("Failed to create .obakignore file: {}", e);
        }
        return Ok(HttpResponse::Ok().json(IgnoreList {
            entries: vec![],
        }));
    }

    match fs::read_to_string(&ignore_path).await {
        Ok(content) => {
            let mut entries = Vec::new();

            for line in content.lines() {
                let line = line.trim();

                // Skip empty lines
                if line.is_empty() {
                    continue;
                }

                // Handle comments
                if line.starts_with('#') {
                    // Skip header comments
                    continue;
                }

                // Check if line has inline comment
                if let Some(pos) = line.find('#') {
                    let pattern = line[..pos].trim().to_string();
                    let comment = line[pos + 1..].trim().to_string();
                    if !pattern.is_empty() {
                        entries.push(IgnoreEntry {
                            pattern,
                            comment: Some(comment),
                        });
                    }
                } else {
                    entries.push(IgnoreEntry {
                        pattern: line.to_string(),
                        comment: None,
                    });
                }
            }

            Ok(HttpResponse::Ok().json(IgnoreList { entries }))
        }
        Err(e) => {
            error!("Failed to read .obakignore file: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to read ignore list"
            })))
        }
    }
}

/// PUT /api/server/:id/backups/ignore - Update the .obakignore file
#[put("/ignore")]
async fn update_ignore_list(
    server_id: web::Path<String>,
    request: web::Json<IgnoreList>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    let ignore_path = server.get_directory_path().join(".obakignore");

    // Build file content
    let mut content = String::from("# Backup ignore patterns\n");
    content.push_str("# Patterns follow .gitignore syntax\n\n");

    for entry in &request.entries {
        if entry.pattern.trim().is_empty() {
            continue;
        }

        if let Some(comment) = &entry.comment {
            if !comment.trim().is_empty() {
                content.push_str(&format!("{}  # {}\n", entry.pattern, comment));
            } else {
                content.push_str(&format!("{}\n", entry.pattern));
            }
        } else {
            content.push_str(&format!("{}\n", entry.pattern));
        }
    }

    match fs::write(&ignore_path, content).await {
        Ok(_) => {
            info!("Updated .obakignore for server '{}'", server.name);
            Ok(HttpResponse::Ok().json(json!({
                "message": "Ignore list updated successfully"
            })))
        }
        Err(e) => {
            error!("Failed to write .obakignore file: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to update ignore list"
            })))
        }
    }
}

/// DELETE /api/server/:id/backups/schedules/:scheduleId - Delete a backup schedule
#[delete("/schedules/{schedule_id}")]
async fn delete_schedule(
    path: web::Path<(String, i64)>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let (server_id, schedule_id) = path.into_inner();
    let server_id = decode_single(server_id)?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    // Verify server exists and user has access
    let _server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    let pool = app_db::open_pool().await?;

    match backup_db::delete_schedule(schedule_id, server_id as i64, &pool).await {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::Ok().json(json!({
                    "message": "Schedule deleted successfully"
                })))
            } else {
                Ok(HttpResponse::NotFound().json(json!({
                    "error": "Schedule not found"
                })))
            }
        }
        Err(e) => {
            error!("Failed to delete schedule: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to delete schedule"
            })))
        }
    }
}
