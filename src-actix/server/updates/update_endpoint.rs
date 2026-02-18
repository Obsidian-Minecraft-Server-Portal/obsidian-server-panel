use super::{update_checker::UpdateChecker, update_service::UpdateService};
use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::UserRequestExt;
use crate::server::server_data::ServerData;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use anyhow::anyhow;
use log::{error, info};
use serde_hash::hashids::decode_single;
use serde_json::json;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/updates")
            .service(check_updates)
            .service(apply_update)
            .service(rollback_update)
            .service(list_backups),
    );
}

/// GET /api/server/:id/updates/check - Check if updates are available
#[get("/check")]
async fn check_updates(
    server_id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    info!(
        "Checking for updates for server '{}' (ID: {})",
        server.name, server_id
    );

    match UpdateChecker::check_for_updates(&server).await {
        Ok(Some(update_info)) => {
            // Update database with latest version info
            let pool = crate::database::get_pool();
            let now = chrono::Utc::now().timestamp();

            sqlx::query(&*crate::database::sql(
                r#"
                UPDATE servers
                SET update_available = 1,
                    latest_version = ?,
                    last_update_check = ?
                WHERE id = ?
                "#
            ))
            .bind(&update_info.latest_version)
            .bind(now)
            .bind(server_id as i64)
            .execute(pool)
            .await?;

            info!(
                "Update available for server '{}': {} -> {}",
                server.name, update_info.current_version, update_info.latest_version
            );

            Ok(HttpResponse::Ok().json(update_info))
        }
        Ok(None) => {
            // No update available, update last check timestamp
            let pool = crate::database::get_pool();
            let now = chrono::Utc::now().timestamp();

            sqlx::query(&*crate::database::sql(
                r#"
                UPDATE servers
                SET update_available = 0,
                    latest_version = NULL,
                    last_update_check = ?
                WHERE id = ?
                "#
            ))
            .bind(now)
            .bind(server_id as i64)
            .execute(pool)
            .await?;

            info!(
                "Server '{}' is up to date",
                server.name
            );

            Ok(HttpResponse::Ok().json(json!({
                "update_available": false,
                "message": "Server is up to date"
            })))
        }
        Err(e) => {
            error!("Failed to check for updates: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to check for updates: {}", e)
            })))
        }
    }
}

/// POST /api/server/:id/updates/apply - Apply available update
#[post("/apply")]
async fn apply_update(
    server_id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    info!("Applying update for server '{}' (ID: {})", server.name, server_id);

    // Check for updates first
    let update_info = match UpdateChecker::check_for_updates(&server).await? {
        Some(info) => info,
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "No updates available"
            })));
        }
    };

    // Apply the update
    match UpdateService::apply_update(&mut server, &update_info).await {
        Ok(_) => {
            info!(
                "Update applied successfully for server '{}': {}",
                server.name, update_info.latest_version
            );

            Ok(HttpResponse::Ok().json(json!({
                "message": "Update applied successfully",
                "new_version": update_info.latest_version
            })))
        }
        Err(e) => {
            error!("Failed to apply update: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to apply update: {}", e)
            })))
        }
    }
}

/// POST /api/server/:id/updates/rollback - Rollback to previous version
#[post("/rollback")]
async fn rollback_update(
    server_id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    info!("Rolling back update for server '{}' (ID: {})", server.name, server_id);

    match UpdateService::rollback_update(&mut server).await {
        Ok(_) => {
            info!(
                "Rollback successful for server '{}'",
                server.name
            );

            Ok(HttpResponse::Ok().json(json!({
                "message": "Rolled back to previous version successfully"
            })))
        }
        Err(e) => {
            error!("Failed to rollback: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to rollback: {}", e)
            })))
        }
    }
}

/// GET /api/server/:id/updates/backups - List available backup jars
#[get("/backups")]
async fn list_backups(
    server_id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found"
            })));
        }
    };

    match UpdateService::list_backup_jars(&server).await {
        Ok(backups) => Ok(HttpResponse::Ok().json(json!({
            "backups": backups
        }))),
        Err(e) => {
            error!("Failed to list backups: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to list backups"
            })))
        }
    }
}
