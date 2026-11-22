use actix_web::{HttpResponse, get, web};
use serde::Serialize;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};
use utoipa::ToSchema;

use obsidian_common::CommonError;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

const MB: u64 = 1024 * 1024;
const OS_RESERVE_MB: u64 = 1024;
const RECOMMENDED_SHARE_PERCENT: u64 = 75;
const RECOMMENDED_CAP_MB: u64 = 8192;

#[derive(Debug, Serialize, ToSchema)]
pub struct MemoryLimits {
	pub total_mb: u64,
	pub usable_mb: u64,
	pub recommended_mb: u64,
}

fn limits(total_bytes: u64) -> MemoryLimits {
	let total_mb = total_bytes / MB;
	let usable_mb = total_mb.saturating_sub(OS_RESERVE_MB);
	let recommended_mb = (usable_mb * RECOMMENDED_SHARE_PERCENT / 100).min(RECOMMENDED_CAP_MB);
	MemoryLimits { total_mb, usable_mb, recommended_mb }
}

#[utoipa::path(
	get,
	path = "/api/v1/system/memory",
	tag = "system",
	responses((status = 200, description = "Memory the host has and the limits a server should respect", body = MemoryLimits))
)]
#[get("/system/memory")]
pub async fn memory_limits(_user: AuthUser, _state: web::Data<AppState>) -> ApiResult<HttpResponse> {
	let system = System::new_with_specifics(RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()))
		.map_err(|error| CommonError::Other(error.to_string()))?;
	Ok(HttpResponse::Ok().json(limits(system.total_memory())))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(memory_limits);
}
