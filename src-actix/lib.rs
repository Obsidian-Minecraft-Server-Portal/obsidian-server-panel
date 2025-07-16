use actix_util::asset_endpoint::AssetsAppConfig;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use anyhow::Result;
use log::*;
use serde_json::json;
use vite_actix::start_vite_server;

mod actix_util;
mod app_db;
mod authentication;

pub static DEBUG: bool = cfg!(debug_assertions);
const PORT: u16 = 8080;

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder().filter_level(if DEBUG { LevelFilter::Debug } else { LevelFilter::Info }).format_timestamp(None).init();
    // setup serde hashids
    serde_hash::hashids::SerdeHashOptions::new().with_min_length(16).build();

    app_db::initialize_databases().await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096).error_handler(|err, _req| {
                let error = json!({ "error": format!("{}", err) });
                actix_web::error::InternalError::from_response(err, HttpResponse::BadRequest().json(error)).into()
            }))
            .service(web::scope("api").configure(authentication::configure))
            .configure_frontend_routes()
    })
    .workers(4)
    .bind(format!("0.0.0.0:{port}", port = PORT))?
    .run();

    info!("Starting {} server at http://127.0.0.1:{}...", if DEBUG { "development" } else { "production" }, PORT);

    if DEBUG {
        start_vite_server().expect("Failed to start vite server");
    }

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}
