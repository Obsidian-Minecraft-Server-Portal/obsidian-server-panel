use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::UserRequestExt;
use crate::updater::updater_service::UpdateService;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::{Arc, OnceLock};

// Global update service instance using OnceLock
static UPDATE_SERVICE: OnceLock<Arc<UpdateService>> = OnceLock::new();

fn get_update_service() -> &'static Arc<UpdateService> {
    UPDATE_SERVICE.get_or_init(|| Arc::new(UpdateService::new()))
}

#[get("")]
pub async fn get_current_version(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    
    // Only admins can check for updates
    if !user.is_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "You don't have permission to check for updates"
        })));
    }

    let service = get_update_service();
    let response = json!({
        "current_version": service.updater.current_version(),
        "status": "ready"
    });

    Ok(HttpResponse::Ok().json(response))
}

#[get("check")]
pub async fn check_for_updates(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    
    // Only admins can check for updates
    if !user.is_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "You don't have permission to check for updates"
        })));
    }

    let service = get_update_service();
    match service.check_updates().await {
        Ok(response) => Ok(response),
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to check for updates".to_string(),
                "message": e.to_string()
            })))
        }
    }
}

#[post("perform")]
pub async fn perform_update(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    
    // Only admins can perform updates
    if !user.is_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "You don't have permission to perform updates"
        })));
    }

    let service = get_update_service();
    match service.perform_update().await {
        Ok(response) => Ok(response),
        Err(e) => {
            log::error!("Failed to perform update: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to perform update".to_string(),
                "message": e.to_string()
            })))
        }
    }
}

#[get("status")]
pub async fn get_update_status(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    
    // Only admins can check update status
    if !user.is_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "You don't have permission to check update status"
        })));
    }

    let service = get_update_service();
    match service.get_status().await {
        Ok(response) => Ok(response),
        Err(e) => {
            log::error!("Failed to get update status: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get update status".to_string(),
                "message": e.to_string()
            })))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/updater")
            .service(get_current_version)
            .service(check_for_updates)
            .service(perform_update)
            .service(get_update_status)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}