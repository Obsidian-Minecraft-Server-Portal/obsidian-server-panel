use crate::actix_util::http_error::Result;
use actix_web::{get, post, web, HttpResponse, Responder};
use curseforge::SearchBuilder;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub query: Option<String>,
    pub game_version: Option<String>,
    pub mod_loader_type: Option<u32>,
    pub category_id: Option<u32>,
    pub page_size: Option<u32>,
    pub index: Option<u32>,
    pub sort_field: Option<u32>,
    pub sort_order: Option<String>,
}

#[derive(Deserialize)]
pub struct IdsQuery {
    pub ids: String,
}

impl SearchQuery {
    fn to_builder(&self) -> SearchBuilder {
        let mut builder = SearchBuilder::new();

        if let Some(ref q) = self.query {
            builder = builder.query(q.as_str());
        }
        if let Some(ref gv) = self.game_version {
            builder = builder.game_version(gv.as_str());
        }
        if let Some(ml) = self.mod_loader_type {
            builder = builder.mod_loader_type(ml);
        }
        if let Some(cat) = self.category_id {
            builder = builder.category_id(cat);
        }
        if let Some(ps) = self.page_size {
            builder = builder.page_size(ps);
        }
        if let Some(idx) = self.index {
            builder = builder.index(idx);
        }
        if let Some(sf) = self.sort_field {
            builder = builder.sort_field(sf);
        }
        if let Some(ref so) = self.sort_order {
            builder = builder.sort_order(so.as_str());
        }

        builder
    }
}

#[get("/search")]
async fn search(query: web::Query<SearchQuery>) -> Result<impl Responder> {
    let client = super::get_client();
    let params = query.to_builder().build();
    let result = client.search(&params).await.map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/search/modpacks")]
async fn search_modpacks(query: web::Query<SearchQuery>) -> Result<impl Responder> {
    let client = super::get_client();
    let params = query.to_builder().build();
    let result = client
        .search_modpacks(&params)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/mod/{id}")]
async fn get_mod(id: web::Path<u32>) -> Result<impl Responder> {
    let client = super::get_client();
    let m = client
        .get_mod(*id)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(m))
}

#[get("/mod/{mod_id}/files")]
async fn get_mod_files(mod_id: web::Path<u32>) -> Result<impl Responder> {
    let client = super::get_client();
    let files = client
        .get_mod_files(*mod_id)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(files))
}

#[get("/mod/{mod_id}/files/{file_id}")]
async fn get_mod_file(path: web::Path<(u32, u64)>) -> Result<impl Responder> {
    let (mod_id, file_id) = path.into_inner();
    let client = super::get_client();
    let file = client
        .get_mod_file(mod_id, file_id)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(file))
}

#[get("/mods")]
async fn get_mods(query: web::Query<IdsQuery>) -> Result<impl Responder> {
    let ids: Vec<u32> = serde_json::from_str(&query.ids)
        .map_err(|e| anyhow::anyhow!("Invalid IDs JSON array: {e}"))?;
    let client = super::get_client();
    let mods = client
        .get_mods(&ids)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(mods))
}

#[get("/categories")]
async fn get_categories() -> Result<impl Responder> {
    let client = super::get_client();
    let categories = client
        .get_categories()
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(categories))
}

#[post("/cache/clear")]
async fn clear_cache() -> Result<impl Responder> {
    let client = super::get_client();
    client.clear_cache().await;
    Ok(HttpResponse::Ok().json(json!({"message": "Cache cleared"})))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/curseforge")
            .service(search)
            .service(search_modpacks)
            .service(get_mod)
            .service(get_mod_files)
            .service(get_mod_file)
            .service(get_mods)
            .service(get_categories)
            .service(clear_cache)
            .default_service(actix_web::web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found",
                }))
            })),
    );
}
