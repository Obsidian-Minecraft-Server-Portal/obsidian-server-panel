use actix_web::{HttpResponse, get, web};
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
	pub status: &'static str,
	pub version: &'static str,
	pub build: &'static str,
	pub setup_required: bool,
	pub smtp_enabled: bool,
	pub running_instances: usize,
	pub sftp_port: Option<u16>,
	pub updater_enabled: bool,
}

#[utoipa::path(
	get,
	path = "/api/v1/health",
	tag = "system",
	responses((status = 200, description = "Panel health", body = HealthResponse))
)]
#[get("/health")]
pub async fn health(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(HealthResponse {
		status: "ok",
		version: env!("CARGO_PKG_VERSION"),
		build: env!("BUILD"),
		setup_required: state.auth.setup_required().await?,
		smtp_enabled: crate::email::effective_smtp(&state).await?.is_some(),
		running_instances: state.instances.running_count(),
		sftp_port: (state.config.sftp_port != 0).then_some(state.config.sftp_port),
		updater_enabled: cfg!(feature = "updater"),
	}))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(health);
}
