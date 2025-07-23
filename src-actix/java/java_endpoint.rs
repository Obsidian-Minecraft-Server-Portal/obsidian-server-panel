use crate::actix_util::http_error::Result;
use crate::java::versions::JavaVersion;
use actix_web::{delete, get, web, HttpResponse, Responder};
use actix_web_lab::sse;
use log::error;
use serde_json::json;
use std::time::Duration;

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

#[get("/install/{runtime}")]
pub async fn install_java_version(runtime: web::Path<String>) -> Result<impl Responder> {
    let (sender, receiver) = tokio::sync::mpsc::channel(10);
    tokio::spawn(async move {
        let runtime = runtime.into_inner();
        if let Ok(version) = JavaVersion::from_runtime(&runtime).await {
            if let Err(e) = version.install(sender.clone()).await {
                error!("Error installing java version: {}", e);
                let data = json!({"message": "Error installing java version", "stacktrace": e.to_string()});
                let message = serde_json::to_string(&data).unwrap();
                let event_data = sse::Data::new(message).event("error");
                sender.try_send(event_data.into()).unwrap();
            }
        }
    });
    Ok(sse::Sse::from_infallible_receiver(receiver).with_keep_alive(Duration::from_secs(10)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/java")
            .service(install_java_version)
            .service(get_java_versions)
            .service(get_installation_files)
            .service(uninstall_java_version)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
