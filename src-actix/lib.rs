mod auth;
mod cli;
mod config;
mod csrf;
mod email;
mod error;
mod openapi;
mod routes;
mod schedule_executor;
mod service;
mod sftp_auth;
mod state;
mod util;
mod ws;

#[cfg(feature = "tray")]
mod tray;
#[cfg(feature = "updater")]
mod updater;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use clap::Parser;
use color_eyre::Result;
use obsidian_sftp::SftpServer;
use serde_json::json;
use tokio::sync::oneshot;
use tracing::{debug, info};
use util::asset_endpoint::AssetsAppConfig;
use vite_actix::start_vite_server;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::service::ServiceAction;
use crate::sftp_auth::PanelSftpAuth;
use crate::state::AppState;

pub static DEBUG: bool = cfg!(debug_assertions);
const FLUSH_INTERVAL: Duration = Duration::from_secs(30);
const JSON_BODY_LIMIT: usize = 2 * 1024 * 1024;
const LOGS_DIR: &str = "logs";
const SFTP_HOST_KEY_FILE: &str = "sftp_host_key";

pub fn main() -> Result<()> {
	color_eyre::install()?;
	let command = Cli::parse().command.unwrap_or(Command::Serve { headless: false });
	match command {
		Command::Serve { headless } => {
			setup_logging(Some(LOGS_DIR.into()))?;
			let config = Config::load()?;
			serve_foreground(config, headless)
		}
		Command::Runner { instance } => {
			setup_logging(Some(format!("{LOGS_DIR}/runner-{instance}")))?;
			Ok(runtime()?.block_on(obsidian_runner::run(instance))?)
		}
		Command::Service { action: ServiceAction::Run } => {
			#[cfg(feature = "service")]
			Config::override_data_dir(service::service_root()?);
			setup_logging(Some(LOGS_DIR.into()))?;
			service::dispatch(|shutdown| serve_blocking(Config::load()?, shutdown))
		}
		Command::Service { action } => {
			setup_logging(None)?;
			service::run(action)
		}
		Command::ResetPassword { username } => {
			setup_logging(None)?;
			runtime()?.block_on(cli::reset_password::run(&username))
		}
	}
}

fn setup_logging(relative: Option<String>) -> Result<()> {
	let dir = relative.map(|relative| Config::data_dir().join(relative));
	util::logging::setup_logging(dir.as_deref())
}

fn runtime() -> Result<tokio::runtime::Runtime> {
	Ok(tokio::runtime::Builder::new_multi_thread().enable_all().build()?)
}

#[cfg(feature = "tray")]
fn serve_foreground(config: Config, headless: bool) -> Result<()> {
	if headless {
		return serve_blocking(config, oneshot::channel().1);
	}
	let url = panel_url(&config);
	tray::run(&url, move |shutdown| serve_blocking(config, shutdown))
}

#[cfg(not(feature = "tray"))]
fn serve_foreground(config: Config, _headless: bool) -> Result<()> {
	serve_blocking(config, oneshot::channel().1)
}

#[cfg(feature = "tray")]
fn panel_url(config: &Config) -> String {
	if let Some(url) = &config.public_url {
		return url.clone();
	}
	let host = match config.bind.as_str() {
		"0.0.0.0" | "::" => "localhost",
		bind => bind,
	};
	format!("http://{host}:{}", config.port)
}

fn serve_blocking(config: Config, shutdown: oneshot::Receiver<()>) -> Result<()> {
	actix_web::rt::System::new().block_on(serve(config, shutdown))
}

async fn serve(config: Config, shutdown: oneshot::Receiver<()>) -> Result<()> {
	let (bind, port) = (config.bind.clone(), config.port);
	let (sftp_ip, sftp_port): (IpAddr, u16) = (config.bind.parse()?, config.sftp_port);
	let state = AppState::new(config).await?;
	routes::settings::seed_from_config(&state).await?;
	let flusher = state.db.spawn_flusher(FLUSH_INTERVAL);
	if let Err(error) = state.instances.reattach_all().await {
		tracing::error!(%error, "failed to reattach running servers");
	}
	state.scheduler.spawn();
	let instances = Arc::clone(&state.instances);
	actix_web::rt::spawn(async move {
		if let Err(error) = instances.start_auto_start().await {
			tracing::error!(%error, "auto-start failed");
		}
	});
	let sftp = start_sftp(&state, SocketAddr::new(sftp_ip, sftp_port)).await?;
	#[cfg(feature = "updater")]
	updater::spawn_background_checks();

	let data = web::Data::new(state.clone());
	let server = HttpServer::new(move || {
		App::new()
			.app_data(data.clone())
			.app_data(json_config())
			.wrap(middleware::from_fn(csrf::verify_origin))
			.wrap(middleware::Logger::default())
			.configure(routes::configure)
			.configure(openapi::configure)
			.configure_frontend_routes()
	})
	.workers(4)
	.bind((bind.as_str(), port))?
	.run();
	let handle = server.handle();
	actix_web::rt::spawn(async move {
		if shutdown.await.is_ok() {
			handle.stop(true).await;
		}
	});

	info!(
		"Starting {} server at http://{bind}:{port}...",
		if DEBUG { "development" } else { "production" },
	);
	let vite = DEBUG
		.then(start_vite_server)
		.and_then(|started| started.inspect_err(|error| tracing::error!(%error, "failed to start vite")).ok());

	let result = server.await;
	if let Some(mut vite) = vite {
		let _ = vite.kill();
		let _ = vite.wait();
	}
	if let Some(sftp) = sftp {
		sftp.stop();
	}
	if cfg!(feature = "docker") {
		info!("stopping running servers before the container exits");
		state.instances.stop_all().await;
	}
	let flushed = state.db.flush().await;
	flusher.abort();
	flushed?;
	debug!("Server stopped");
	Ok(result?)
}

async fn start_sftp(state: &AppState, addr: SocketAddr) -> Result<Option<SftpServer>> {
	if addr.port() == 0 {
		return Ok(None);
	}
	let host_key = state.paths.data_dir.join(SFTP_HOST_KEY_FILE);
	let auth = Arc::new(PanelSftpAuth::new(state.clone()));
	let server = SftpServer::start(addr, host_key, auth).await?;
	info!("SFTP server listening on {}", server.addr());
	Ok(Some(server))
}

fn json_config() -> web::JsonConfig {
	web::JsonConfig::default().limit(JSON_BODY_LIMIT).error_handler(|err, _req| {
		let body = json!({ "error": err.to_string() });
		actix_web::error::InternalError::from_response(err, HttpResponse::BadRequest().json(body)).into()
	})
}
