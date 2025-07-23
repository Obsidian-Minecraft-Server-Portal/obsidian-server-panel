use crate::actix_util::http_error::Result;
use crate::java::versions::JavaVersion;
use actix_web::{delete, get, web, HttpResponse, Responder};
use serde_json::json;

#[get("/versions")]
pub async fn get_java_versions() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(json!(JavaVersion::list().await?)))
}

#[get("/versions/{runtime}/files")]
pub async fn get_installation_files(runtime: web::Path<String>) -> impl Responder {
    let runtime = runtime.into_inner();
    match JavaVersion::from_runtime(&runtime).await {
        Ok(v) => match v.get_installation_files().await {
            Ok(files) => HttpResponse::Ok().json(json!(files)),
            Err(e) => HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
        },
        Err(e) => HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
    }
}

#[delete("/versions/{runtime}")]
pub async fn uninstall_java_version(runtime: web::Path<String>) -> impl Responder {
    let runtime = runtime.into_inner();
    match JavaVersion::from_runtime(&runtime).await {
        Ok(v) => match v.uninstall() {
            Ok(_) => HttpResponse::Ok().json(json!({ "message": "Uninstalled" })),
            Err(e) => HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
        },
        Err(e) => HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
    }
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/java").service(get_java_versions).service(get_installation_files).service(uninstall_java_version).default_service(
        web::to(|| async {
            HttpResponse::NotFound().json(json!({
                "error": "API endpoint not found".to_string(),
            }))
        }),
    ));
}
