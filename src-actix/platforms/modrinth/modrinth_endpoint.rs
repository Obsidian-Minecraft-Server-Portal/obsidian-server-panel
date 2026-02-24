use crate::actix_util::http_error::Result;
use actix_web::{get, post, web, HttpResponse, Responder};
use modrinth::SearchBuilder;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub facets: Option<String>,
    pub index: Option<String>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct IdsQuery {
    pub ids: String,
}

#[get("/search")]
async fn search(query: web::Query<SearchQuery>) -> Result<impl Responder> {
    let client = super::get_client();

    let mut builder = SearchBuilder::new();

    if let Some(ref q) = query.query {
        builder = builder.query(q.as_str());
    }

    if let Some(ref facets_str) = query.facets {
        let facets: Vec<Vec<String>> = serde_json::from_str(facets_str)
            .map_err(|e| anyhow::anyhow!("Invalid facets JSON: {e}"))?;
        for group in facets {
            builder = builder.facet_or(group);
        }
    }

    if let Some(ref index) = query.index {
        let search_index = match index.as_str() {
            "relevance" => modrinth::models::SearchIndex::Relevance,
            "downloads" => modrinth::models::SearchIndex::Downloads,
            "follows" => modrinth::models::SearchIndex::Follows,
            "newest" => modrinth::models::SearchIndex::Newest,
            "updated" => modrinth::models::SearchIndex::Updated,
            _ => modrinth::models::SearchIndex::Relevance,
        };
        builder = builder.index(search_index);
    }

    if let Some(offset) = query.offset {
        builder = builder.offset(offset);
    }

    if let Some(limit) = query.limit {
        builder = builder.limit(limit);
    }

    let params = builder.build();
    let result = client.search(&params).await.map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/project/{id}")]
async fn get_project(id: web::Path<String>) -> Result<impl Responder> {
    let client = super::get_client();
    let project = client
        .get_project(&id)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(project))
}

#[get("/project/{id}/versions")]
async fn get_project_versions(id: web::Path<String>) -> Result<impl Responder> {
    let client = super::get_client();
    let versions = client
        .get_project_versions(&id)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(versions))
}

#[get("/version/{id}")]
async fn get_version(id: web::Path<String>) -> Result<impl Responder> {
    let client = super::get_client();
    let version = client
        .get_version(&id)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(version))
}

#[get("/projects")]
async fn get_projects(query: web::Query<IdsQuery>) -> Result<impl Responder> {
    let ids: Vec<String> = serde_json::from_str(&query.ids)
        .map_err(|e| anyhow::anyhow!("Invalid IDs JSON array: {e}"))?;
    let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
    let client = super::get_client();
    let projects = client
        .get_projects(&id_refs)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(projects))
}

#[get("/tags/categories")]
async fn get_categories() -> Result<impl Responder> {
    let client = super::get_client();
    let categories = client
        .get_categories()
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(categories))
}

#[get("/tags/game-versions")]
async fn get_game_versions() -> Result<impl Responder> {
    let client = super::get_client();
    let versions = client
        .get_game_versions()
        .await
        .map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(versions))
}

#[get("/tags/loaders")]
async fn get_loaders() -> Result<impl Responder> {
    let client = super::get_client();
    let loaders = client.get_loaders().await.map_err(anyhow::Error::from)?;
    Ok(HttpResponse::Ok().json(loaders))
}

#[post("/cache/clear")]
async fn clear_cache() -> Result<impl Responder> {
    let client = super::get_client();
    client.clear_cache().await;
    Ok(HttpResponse::Ok().json(json!({"message": "Cache cleared"})))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/modrinth")
            .service(search)
            .service(get_project)
            .service(get_project_versions)
            .service(get_version)
            .service(get_projects)
            .service(get_categories)
            .service(get_game_versions)
            .service(get_loaders)
            .service(clear_cache)
            .default_service(actix_web::web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found",
                }))
            })),
    );
}
