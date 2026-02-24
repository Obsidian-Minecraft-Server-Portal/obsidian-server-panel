use crate::server::server_data::ServerData;
use actix_util::asset_endpoint::AssetsAppConfig;
use actix_web::Responder;
use actix_web::{App, HttpResponse, HttpServer, get, middleware, web};
use anyhow::Result;
use clap::Parser;
use log::*;
use obsidian_upnp::{UpnpManager, PortMappingProtocol};
use serde_json::json;
use std::env::set_current_dir;
use obsidian_scheduler::timer_trait::Timer;
use vite_actix::proxy_vite_options::ProxyViteOptions;
use vite_actix::start_vite_server;

mod actions;
mod actix_util;
mod app_db;
mod authentication;
mod broadcast;
mod command_line_args;
pub mod database;
mod forge_endpoint;
mod host_info;
mod java;
mod notifications;
mod server;
mod settings;
mod updater;
mod platforms;

pub static DEBUG: bool = cfg!(debug_assertions);
static ICON: &[u8] = include_bytes!("../resources/logo/icon.ico");
pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder().filter_level(if DEBUG { LevelFilter::Debug } else { LevelFilter::Info }).format_timestamp(None).init();
    info!("Starting Obsidian Minecraft Server Panel...");
    let args = command_line_args::CommandLineArgs::parse();

    #[cfg(debug_assertions)]
    {
        ProxyViteOptions::new().disable_logging().build()?;
        std::thread::spawn(|| {
            loop {
                info!("Starting Vite server in development mode...");
                let status = start_vite_server().expect("Failed to start vite server").wait().expect("Vite server crashed!");
                if !status.success() {
                    error!("The vite server has crashed!");
                } else {
                    break;
                }
            }
        });

        // setup serde hashids
        serde_hash::hashids::SerdeHashOptions::new().with_min_length(16).with_salt("obsidian-server-panel").build();
        let dev_env_path = "./target/dev-env";
        std::fs::create_dir_all(dev_env_path)?;
        set_current_dir(dev_env_path)?;
    }
    #[cfg(not(debug_assertions))]
    serde_hash::hashids::SerdeHashOptions::new().with_min_length(16).build();

    // Initialize settings path
    settings::initialize_settings_path();

    // Create shared database pool (stored globally)
    let pool = database::init_pool().await?;

    tokio::spawn(async {
        let pool = database::get_pool();
        let result: Result<()> = async {
            app_db::initialize_databases(pool).await?;
            ServerData::initialize_servers(pool).await?;

            // Only refresh Java version map if expired (older than 1 day)
            let is_expired = java::is_version_map_expired(pool).await?;

            if is_expired {
                java::refresh_java_minecraft_version_map().await?;
            }
            Ok(())
        }
        .await;

        if let Err(e) = result {
            error!("Database initialization failed: {}", e);
        }
    });

    // Start the scheduler to refresh Java Minecraft version map daily
    let java_scheduler = java::start_scheduler();
    java_scheduler.start().await?;

    // Start the backup scheduler
    tokio::spawn(async {
        let pool = database::get_pool();
        let result: Result<()> = async {
            let mut backup_scheduler = server::backups::BackupScheduler::new(pool.clone());
            backup_scheduler.start().await?;

            // Keep the scheduler running
            loop {
                if !backup_scheduler.is_running().await {
                    warn!("Backup scheduler stopped, restarting...");
                    backup_scheduler.start().await?;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        }
        .await;

        if let Err(e) = result {
            error!("Backup scheduler failed: {}", e);
        }
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::JsonConfig::default().limit(4096).error_handler(|err, _req| {
                let error = json!({ "error": format!("{}", err) });
                actix_web::error::InternalError::from_response(err, HttpResponse::BadRequest().json(error)).into()
            }))
            .service(get_icon)
            .service(
                web::scope("api").configure(host_info::configure).configure(authentication::configure).service(
                    web::scope("")
                        .wrap(authentication::AuthenticationMiddleware)
                        .configure(actions::configure)
                        .configure(java::configure)
                        .configure(forge_endpoint::configure)
                        .configure(server::configure)
                        .configure(settings::configure)
                        .configure(updater::configure)
                        .configure(broadcast::updates_endpoint::configure)
                        .configure(platforms::configure)
                    ,
                ),
            )
            .configure_frontend_routes()
    })
    .workers(std::thread::available_parallelism()?.get())
    .bind(format!("0.0.0.0:{port}", port = args.port))?
    .run();

    info!("Starting {} server at http://127.0.0.1:{}...", if DEBUG { "development" } else { "production" }, args.port);

    if args.forward_webpanel {
        if let Err(e) = UpnpManager::global()
            .add_port(
                args.port,
                "Obsidian Minecraft Server Panel".into(),
                PortMappingProtocol::TCP,
            )
            .await
        {
            error!("Failed to open UPnP port {}: {}", args.port, e);
        } else {
            info!("Successfully opened UPnP port {}", args.port);
        }
    }

    let stop_result = server.await;
    debug!("Server stopped");

    // Close all UPnP ports
    if let Err(e) = UpnpManager::global().remove_all_ports().await {
        error!("Failed to clean up UPnP ports: {}", e);
    }

    Ok(stop_result?)
}

#[get("/favicon.ico")]
pub async fn get_icon() -> impl Responder {
    HttpResponse::Ok().content_type("image/x-icon").body(ICON)
}
