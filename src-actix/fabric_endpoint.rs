use crate::actix_util::http_error::Result;
use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/versions")]
pub async fn get_fabric_versions() -> Result<impl Responder> {
    let client = fabric_loader::FabricClient::new();
    let versions = client
        .get_versions()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(HttpResponse::Ok().json(versions))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/fabric")
            .service(get_fabric_versions)
            .default_service(actix_web::web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
