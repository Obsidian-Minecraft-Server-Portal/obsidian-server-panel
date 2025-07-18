use crate::actix_util::http_error::Result;
use crate::authentication::user_permissions::PermissionFlag;
use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct ServerInfo {
    version: String,
    is_development: bool,
    has_admin_user: bool,
}

impl ServerInfo {
    pub async fn get() -> anyhow::Result<Self> {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let is_development = cfg!(debug_assertions);
        let pool = crate::app_db::open_pool().await?;
        let has_admin_user = !crate::authentication::auth_data::UserData::get_users_with_permissions(PermissionFlag::Admin, &pool).await?.is_empty();
        pool.close().await; // Close the database connection after use

        Ok(Self { version, is_development, has_admin_user })
    }
}

#[get("")]
pub async fn get_server_info() -> Result<impl Responder> {
    let server_info = ServerInfo::get().await?;
    Ok(HttpResponse::Ok().json(server_info))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/").service(get_server_info).default_service(actix_web::web::to(|| async {
        HttpResponse::NotFound().json(json!({
            "error": "API endpoint not found".to_string(),
        }))
    })));
}
