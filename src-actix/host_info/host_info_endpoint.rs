use crate::actix_util::http_error::Result;
use crate::host_info::host_info_data::HostInfo;
use crate::host_info::host_resource_data::HostResourceData;
use actix_web::{get, HttpResponse, Responder};
use actix_web_lab::sse;
use log::error;
use serde_json::json;

#[get("")]
pub async fn get_host_info() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(HostInfo::get().await?))
}

#[get("resources")]
pub async fn get_resources() -> impl Responder {
    let (sender, receiver) = tokio::sync::mpsc::channel(2);

    tokio::spawn(async move {
        if let Err(e) = HostResourceData::fetch_continuously(sender, None).await {
            error!("Failed to fetch host resource data: {}", e)
        }
    });

    sse::Sse::from_infallible_receiver(receiver).with_keep_alive(std::time::Duration::from_secs(10))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/").service(get_host_info).service(get_resources).default_service(actix_web::web::to(|| async {
        HttpResponse::NotFound().json(json!({
            "error": "API endpoint not found".to_string(),
        }))
    })));
}
