use crate::actix_util::http_error::Result;
use actix_web::{HttpResponse, Responder, get, web};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[get("/search")]
async fn search(query: web::Query<SearchQuery>) -> Result<impl Responder> {
    let result = super::search(query.q.as_deref()).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/modpack/{slug}")]
async fn get_modpack(path: web::Path<String>) -> Result<impl Responder> {
    let modpack = super::fetch_modpack(&path).await?;
    Ok(HttpResponse::Ok().json(modpack))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/technic")
            .service(search)
            .service(get_modpack)
            .default_service(actix_web::web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found",
                }))
            })),
    );
}
