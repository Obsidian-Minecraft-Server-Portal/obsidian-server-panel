use crate::app_db::open_pool;
use crate::server::server_data::ServerData;
use actix_util::asset_endpoint::AssetsAppConfig;
use actix_web::Responder;
use actix_web::{App, HttpResponse, HttpServer, get, middleware, web};
use anyhow::Result;
use clap::Parser;
use log::*;
use easy_upnp::{add_ports, delete_ports, PortMappingProtocol, UpnpConfig};
use std::sync::{Mutex, OnceLock};
use serde_json::json;
use std::env::set_current_dir;
use obsidian_scheduler::timer_trait::Timer;
use vite_actix::proxy_vite_options::ProxyViteOptions;
use vite_actix::start_vite_server;

mod actions;
mod actix_util;
mod app_db;
mod authentication;
mod command_line_args;
mod forge_endpoint;
mod host_info;
mod java;
mod server;
mod updater;

pub static DEBUG: bool = cfg!(debug_assertions);
static ICON: &[u8] = include_bytes!("../resources/logo/icon.ico");
static UPNP_PORTS: OnceLock<Mutex<Vec<u16>>> = OnceLock::new();

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

    tokio::spawn(async {
        let result: Result<()> = async {
            let pool = open_pool().await?;
            app_db::initialize_databases(&pool).await?;
            ServerData::initialize_servers(&pool).await?;

            // Only refresh Java version map if expired (older than 1 day)
            let is_expired = java::is_version_map_expired(&pool).await?;
            pool.close().await;

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
        let result: Result<()> = async {
            let pool = open_pool().await?;
            let mut backup_scheduler = server::backups::BackupScheduler::new(pool);
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
                        .configure(updater::configure),
                ),
            )
            .configure_frontend_routes()
    })
    .workers(std::thread::available_parallelism()?.get())
    .bind(format!("0.0.0.0:{port}", port = args.port))?
    .run();

    info!("Starting {} server at http://127.0.0.1:{}...", if DEBUG { "development" } else { "production" }, args.port);

    if args.forward_webpanel {
        let config = UpnpConfig {
            address: None,
            port: args.port,
            protocol: PortMappingProtocol::TCP,
            duration: 0, // 0 means indefinite or default lease time
            comment: "Obsidian Minecraft Server Panel".to_string(),
        };

        for result in add_ports(vec![config]) {
            if let Err(e) = result {
                error!("Failed to open UPnP port {}: {}", args.port, e);
            } else {
                info!("Successfully opened UPnP port {}", args.port);
                match UPNP_PORTS.get_or_init(|| Mutex::new(Vec::new())).lock() {
                    Ok(mut ports) => ports.push(args.port),
                    Err(e) => error!("Failed to store UPnP port (mutex poisoned): {}", e),
                }
            }
        }
    }

    let stop_result = server.await;
    debug!("Server stopped");

    // Close all UPnP ports
    if let Some(upnp_ports) = UPNP_PORTS.get() {
        match upnp_ports.lock() {
            Ok(ports_guard) => {
                let ports = ports_guard.clone();
                drop(ports_guard); // Release lock before blocking operation

                if !ports.is_empty() {
                    let configs: Vec<UpnpConfig> = ports
                        .iter()
                        .map(|&port| UpnpConfig {
                            address: None,
                            port,
                            protocol: PortMappingProtocol::TCP,
                            duration: 0,
                            comment: "Obsidian Port".to_string(),
                        })
                        .collect();

                    for result in delete_ports(configs) {
                        if let Err(e) = result {
                            error!("Failed to close UPnP port: {}", e);
                        } else {
                            debug!("Successfully closed UPnP port");
                        }
                    }

                    // Clear the ports list after attempting to close them
                    if let Ok(mut ports_guard) = upnp_ports.lock() {
                        ports_guard.clear();
                    }
                }
            }
            Err(e) => error!("Failed to access UPnP ports for cleanup (mutex poisoned): {}", e),
        }
    }

    Ok(stop_result?)
}

#[get("/favicon.ico")]
pub async fn get_icon() -> impl Responder {
    HttpResponse::Ok().content_type("image/x-icon").body(ICON)
}
