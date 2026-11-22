use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use actix_web::{HttpResponse, get, post, web};
use chrono::{DateTime, Utc};
use color_eyre::Result;
use obsidian_common::CommonError;
use self_update::ReleaseStatus;
use self_update::backends::github::Update;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const OWNER: &str = "Obsidian-Minecraft-Server-Portal";
const REPO: &str = "obsidian-server-panel";
const BIN_NAME: &str = "obsidian";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const RESTART_DELAY: Duration = Duration::from_secs(1);
const ARCHIVE_PREFIX: &str = if cfg!(all(target_os = "linux", feature = "tray")) {
	"obsidian-desktop-"
} else {
	"obsidian-"
};
const ARCHIVE_EXTENSION: &str = if cfg!(windows) { ".zip" } else { ".tar.gz" };

static LATEST: Mutex<Option<UpdateStatus>> = Mutex::new(None);
static APPLYING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UpdateStatus {
	pub current_version: String,
	pub latest_version: Option<String>,
	pub update_available: bool,
	pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct UpdateQuery {
	#[serde(default)]
	refresh: bool,
}

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct ApplyResponse {
	pub updated: bool,
	pub version: Option<String>,
	pub restarting: bool,
}

fn updater() -> Result<Update> {
	let archive = format!("{ARCHIVE_PREFIX}{}{ARCHIVE_EXTENSION}", self_update::get_target());
	Ok(Update::configure()
		.repo_owner(OWNER)
		.repo_name(REPO)
		.bin_name(BIN_NAME)
		.asset_identifier(archive)
		.current_version(CURRENT_VERSION)
		.show_download_progress(false)
		.show_output(false)
		.no_confirm(true)
		.build()?)
}

fn check() -> Result<UpdateStatus> {
	let (latest_version, update_available) = match updater()?.get_latest_release() {
		Ok(releases) => (releases.latest().map(|release| release.version().to_owned()), releases.is_update_available()?),
		Err(self_update::errors::Error::NotFound { .. }) => (None, false),
		Err(error) => return Err(error.into()),
	};
	Ok(UpdateStatus { current_version: CURRENT_VERSION.to_owned(), latest_version, update_available, checked_at: Utc::now() })
}

fn apply() -> Result<Option<String>> {
	Ok(match updater()?.update_extended()? {
		ReleaseStatus::Updated(release) => Some(release.version().to_owned()),
		_ => None,
	})
}

async fn blocking<T: Send + 'static>(job: fn() -> Result<T>) -> Result<T> {
	tokio::task::spawn_blocking(job).await?
}

fn internal(error: color_eyre::Report) -> ApiError {
	CommonError::Other(format!("{error:#}")).into()
}

async fn refresh() -> Result<UpdateStatus> {
	let status = blocking(check).await?;
	*LATEST.lock().unwrap_or_else(PoisonError::into_inner) = Some(status.clone());
	Ok(status)
}

pub fn spawn_background_checks() {
	tokio::spawn(async {
		let mut ticker = tokio::time::interval(CHECK_INTERVAL);
		loop {
			ticker.tick().await;
			match refresh().await {
				Ok(status) if status.update_available => tracing::info!(
					current = CURRENT_VERSION,
					latest = status.latest_version.as_deref().unwrap_or("unknown"),
					"update available"
				),
				Ok(_) => {}
				Err(error) => tracing::warn!("update check failed: {error:#}"),
			}
		}
	});
}

fn restart() {
	if crate::service::is_windows_service() {
		std::process::exit(1);
	}
	let Err(error) = self_update::restart::restart();
	APPLYING.store(false, Ordering::Release);
	tracing::error!(%error, "failed to restart after update");
}

#[utoipa::path(
	get,
	path = "/api/v1/updates",
	tag = "system",
	params(("refresh" = Option<bool>, Query, description = "Force a new check instead of using the cached result")),
	responses((status = 200, description = "Latest known update status", body = UpdateStatus))
)]
#[get("/updates")]
pub async fn get_updates(_admin: AdminUser, web::Query(query): web::Query<UpdateQuery>) -> ApiResult<HttpResponse> {
	let cached = LATEST.lock().unwrap_or_else(PoisonError::into_inner).clone();
	let status = match cached {
		Some(status) if !query.refresh => status,
		_ => refresh().await.map_err(internal)?,
	};
	Ok(HttpResponse::Ok().json(status))
}

#[utoipa::path(
	post,
	path = "/api/v1/updates/apply",
	tag = "system",
	responses(
		(status = 200, description = "Update result; the panel restarts when updated", body = ApplyResponse),
		(status = 409, description = "An update is already being applied")
	)
)]
#[post("/updates/apply")]
pub async fn apply_update(
	_admin: AdminUser,
	state: web::Data<AppState>,
) -> ApiResult<HttpResponse> {
	if APPLYING.swap(true, Ordering::AcqRel) {
		return Err(CommonError::Conflict("an update is already being applied".into()).into());
	}
	let version = match blocking(apply).await {
		Ok(Some(version)) => version,
		other => {
			APPLYING.store(false, Ordering::Release);
			other.map_err(internal)?;
			return Ok(HttpResponse::Ok().json(ApplyResponse::default()));
		}
	};
	let state = state.into_inner();
	tokio::spawn(async move {
		tokio::time::sleep(RESTART_DELAY).await;
		match state.db.flush().await {
			Ok(()) => restart(),
			Err(error) => {
				APPLYING.store(false, Ordering::Release);
				tracing::error!(%error, "database flush failed, skipping restart after update");
			}
		}
	});
	Ok(HttpResponse::Ok().json(ApplyResponse {
		updated: true,
		version: Some(version),
		restarting: true,
	}))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(get_updates).service(apply_update);
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_updates, apply_update), components(schemas(UpdateStatus, ApplyResponse)))]
pub struct UpdaterApi;
