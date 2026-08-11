use crate::actix_util::http_error::Result;
use actix_web::{HttpResponse, Responder, get, web};
use serde::Deserialize;
use serde_json::{Value, json};

const PACK_FIELDS: &str = "id name safeName description type latestVersion { version minecraftVersion publishedAt }";

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub limit: Option<u32>,
}

#[get("/search")]
async fn search(query: web::Query<SearchQuery>) -> Result<impl Responder> {
    let limit = query.limit.unwrap_or(50).min(100);
    let packs = match query.query.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        Some(q) => {
            let escaped = Value::String(q.to_string()).to_string();
            let data = super::graphql(format!("query {{ searchPacks(first: {limit}, query: {escaped}, field: NAME) {{ {PACK_FIELDS} }} }}")).await?;
            data["searchPacks"].clone()
        }
        None => {
            let data = super::graphql(format!("query {{ packs(first: {limit}) {{ {PACK_FIELDS} }} }}")).await?;
            data["packs"].clone()
        }
    };
    Ok(HttpResponse::Ok().json(packs))
}

#[get("/pack/{safe_name}")]
async fn get_pack(path: web::Path<String>) -> Result<impl Responder> {
    let safe_name = path.into_inner();
    if !super::valid_component(&safe_name) {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "Invalid pack name"})));
    }
    let pack = super::v1(&format!("pack/{safe_name}")).await?;
    Ok(HttpResponse::Ok().json(pack))
}

#[get("/pack/{safe_name}/{version}/manifest")]
async fn get_manifest(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (safe_name, version) = path.into_inner();
    let manifest = super::fetch_manifest(&safe_name, &version).await?;
    Ok(HttpResponse::Ok().json(manifest))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/atlauncher")
            .service(search)
            .service(get_pack)
            .service(get_manifest)
            .default_service(actix_web::web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found",
                }))
            })),
    );
}
