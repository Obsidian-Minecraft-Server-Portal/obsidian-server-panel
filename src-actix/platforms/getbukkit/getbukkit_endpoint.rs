use crate::actix_util::http_error::Result;
use actix_web::{HttpResponse, Responder, get, web};
use serde_json::json;

#[get("/{flavor}/versions")]
async fn versions(path: web::Path<String>) -> Result<impl Responder> {
    let Some(flavor) = super::flavor_name(&path) else {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": format!("Unknown getbukkit flavor: {path}")
        })));
    };
    let versions = super::fetch_versions(flavor).await?;
    let items: Vec<_> = versions
        .into_iter()
        .map(|v| {
            json!({
                "version": v,
                "download_url": super::jar_url(flavor, &v),
            })
        })
        .collect();
    Ok(HttpResponse::Ok().json(items))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/getbukkit").service(versions).default_service(actix_web::web::to(|| async {
        HttpResponse::NotFound().json(json!({
            "error": "API endpoint not found",
        }))
    })));
}
