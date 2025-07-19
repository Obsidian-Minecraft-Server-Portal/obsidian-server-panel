use crate::actix_util::http_error::Result;
use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/versions")]
pub async fn get_forge_versions() -> Result<impl Responder> {
    let request = reqwest::Client::default().get("https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json").build().unwrap();
    let text = reqwest::Client::default().execute(request).await.unwrap().text().await.unwrap();
    Ok(HttpResponse::Ok().content_type("application/json").body(text))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/forge").service(get_forge_versions).default_service(actix_web::web::to(|| async {
        HttpResponse::NotFound().json(json!({
            "error": "API endpoint not found".to_string(),
        }))
    })));
}
