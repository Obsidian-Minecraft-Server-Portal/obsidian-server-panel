use crate::app_db::open_pool;
use crate::server::server_data::ServerData;
use actix_util::asset_endpoint::AssetsAppConfig;
use actix_web::Responder;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use anyhow::Result;
use log::*;
use obsidian_upnp::open_port;
use serde_json::json;
use std::env::set_current_dir;
use vite_actix::start_vite_server;

mod actix_util;
mod app_db;
mod authentication;
mod ddos_middleware;
mod forge_endpoint;
mod host_info;
mod java;
mod server;

pub static DEBUG: bool = cfg!(debug_assertions);
const PORT: u16 = 8080;
static ICON: &[u8] = include_bytes!("../resources/logo/icon.ico");

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder().filter_level(if DEBUG { LevelFilter::Debug } else { LevelFilter::Info }).format_timestamp(None).init();
    // setup serde hashids

    #[cfg(debug_assertions)]
    {
        serde_hash::hashids::SerdeHashOptions::new().with_min_length(16).with_salt("obsidian-server-panel").build();
        let dev_env_path = "./target/dev-env";
        std::fs::create_dir_all(dev_env_path)?;
        set_current_dir(dev_env_path)?;
    }
    #[cfg(not(debug_assertions))]
    serde_hash::hashids::SerdeHashOptions::new().with_min_length(16).build();

    tokio::spawn(async {
        let result: Result<()> = async {
            let pool = open_pool().await?;
            app_db::initialize_databases(&pool).await?;
            ServerData::initialize_servers(&pool).await?;
            pool.close().await;

            Ok(())
        }.await;

        if let Err(e) = result {
            error!("Database initialization failed: {}", e);
        }
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096).error_handler(|err, _req| {
                let error = json!({ "error": format!("{}", err) });
                actix_web::error::InternalError::from_response(err, HttpResponse::BadRequest().json(error)).into()
            }))
            .service(get_icon)
            .service(
                web::scope("api").configure(host_info::configure).configure(authentication::configure).service(
                    web::scope("")
                        .wrap(authentication::AuthenticationMiddleware)
                        .configure(java::configure)
                        .configure(forge_endpoint::configure)
                        .configure(server::configure),
                ),
            )
            .configure_frontend_routes()
    })
    .workers(std::thread::available_parallelism()?.get())
    .bind(format!("0.0.0.0:{port}", port = PORT))?
    .run();

    info!("Starting {} server at http://127.0.0.1:{}...", if DEBUG { "development" } else { "production" }, PORT);

    if DEBUG {
        #[allow(clippy::zombie_processes)]
        start_vite_server().expect("Failed to start vite server");
    }

    open_port!(PORT, "Obsidian Minecraft Server Panel");

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}

#[get("/favicon.ico")]
pub async fn get_icon() -> impl Responder {
    HttpResponse::Ok().content_type("image/x-icon").body(ICON)
}
