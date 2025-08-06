use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::{UserData, UserRequestExt};
use crate::server::backups::backup_data::BackupData;
use crate::server::backups::backup_scheduler;
use crate::server::backups::backup_type::BackupType;
use crate::server::server_data::ServerData;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, post, put, web};
use serde::{Deserialize, Serialize};
use serde_hash::hashids::decode_single;
use serde_json::json;

#[derive(Deserialize)]
struct CreateBackupRequest {
    description: Option<String>,
}

#[derive(Deserialize)]
struct UpdateBackupSettingsRequest {
    backup_enabled: Option<bool>,
    backup_cron: Option<String>,
    backup_type: Option<BackupType>,
    backup_retention: Option<u32>,
}

#[derive(Serialize)]
struct BackupResponse {
    id: u64,
    server_id: u64,
    filename: String,
    backup_type: BackupType,
    file_size: i64,
    file_size_formatted: String,
    created_at: i64,
    created_at_formatted: String,
    description: Option<String>,
}

impl From<BackupData> for BackupResponse {
    fn from(backup: BackupData) -> Self {
        let file_size_formatted = backup.format_file_size();
        let created_at_formatted = backup.format_created_at();

        Self {
            id: backup.id,
            server_id: backup.server_id,
            filename: backup.filename,
            backup_type: backup.backup_type,
            file_size: backup.file_size,
            file_size_formatted,
            created_at: backup.created_at,
            created_at_formatted,
            description: backup.description,
        }
    }
}

#[derive(Serialize)]
struct BackupSettingsResponse {
    backup_enabled: bool,
    backup_cron: String,
    backup_type: BackupType,
    backup_retention: u32,
    is_scheduled: bool,
}

#[get("")]
async fn list_backups(path: web::Path<String>, req: HttpRequest) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    // Parse server_id from the path
    let server_id: u64 = decode_single(server_id)?;

    // Extract authenticated user ID from request extensions
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
        Ok(Some(server)) => match server.list_backups().await {
            Ok(backups) => {
                let backup_responses: Vec<BackupResponse> = backups.into_iter().map(BackupResponse::from).collect();

                Ok(HttpResponse::Ok().json(json!({
                    "backups": backup_responses
                })))
            }
            Err(e) => {
                log::error!("Failed to list backups for server {}: {}", server_id, e);
                Ok(HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to list backups"
                })))
            }
        },
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
async fn create_backup(path: web::Path<String>, req_body: web::Json<CreateBackupRequest>, req: HttpRequest) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;

    // Extract authenticated user ID from request extensions
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
        Ok(Some(server)) => match server.create_backup_with_description(req_body.description.clone()).await {
            Ok(()) => Ok(HttpResponse::Ok().json(json!({
                "message": "Backup created successfully"
            }))),
            Err(e) => {
                log::error!("Failed to create backup for server {}: {}", server_id, e);
                Ok(HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to create backup: {}", e)
                })))
            }
        },
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
async fn delete_backup(path: web::Path<(String, String)>, req: HttpRequest) -> Result<HttpResponse> {
    let (server_id, backup_id) = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;
    let backup_id: u64 = decode_single(backup_id)?;

    // Extract authenticated user ID from request extensions
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
        Ok(Some(server)) => match server.delete_backup(backup_id as i64).await {
            Ok(()) => Ok(HttpResponse::Ok().json(json!({
                "message": "Backup deleted successfully"
            }))),
            Err(e) => {
                log::error!("Failed to delete backup {} for server {}: {}", backup_id, server_id, e);
                Ok(HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to delete backup: {}", e)
                })))
            }
        },
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

    // Extract authenticated user ID from request extensions
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
                backup_type: server.backup_type,
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
async fn update_backup_settings(path: web::Path<String>, req_body: web::Json<UpdateBackupSettingsRequest>, req: HttpRequest) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    let server_id: u64 = decode_single(server_id)?;

    // Extract authenticated user ID from request extensions
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
            // Update server settings
            if let Some(backup_enabled) = req_body.backup_enabled {
                server.backup_enabled = backup_enabled;
            }
            if let Some(backup_cron) = &req_body.backup_cron {
                server.backup_cron = backup_cron.clone();
            }
            if let Some(backup_type) = &req_body.backup_type {
                server.backup_type = backup_type.clone();
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
                            log::error!("Failed to reschedule backup for server {}: {}", server_id, e);
                        }
                    } else if let Err(e) = backup_scheduler::unschedule_server_backup(server_id).await {
                        log::error!("Failed to unschedule backup for server {}: {}", server_id, e);
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
