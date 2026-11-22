use actix_web::{HttpResponse, get, post, web};
use obsidian_auth::Permission;
use obsidian_common::InstanceId;

use super::dto::{CommandRequest, ConsoleQuery, PowerAction, PowerRequest};
use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

const DEFAULT_CONSOLE_LINES: usize = 200;
const MAX_CONSOLE_LINES: usize = 2000;

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/power",
	tag = "instances",
	request_body = PowerRequest,
	responses((status = 202, description = "Power action accepted; returns the new status"))
)]
#[post("/instances/{id}/power")]
pub async fn power(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<PowerRequest>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let action = body.action;
	let permission = match action {
		PowerAction::Start => Permission::POWER_START,
		PowerAction::Stop => Permission::POWER_STOP,
		PowerAction::Restart => Permission::POWER_RESTART,
		PowerAction::Kill => Permission::POWER_KILL,
	};
	require_instance_perm(&state, &user, id, permission).await?;
	match action {
		PowerAction::Start => state.instances.start(id).await?,
		PowerAction::Stop => state.instances.stop(id).await?,
		PowerAction::Restart => state.instances.restart(id).await?,
		PowerAction::Kill => state.instances.kill(id).await?,
	}
	Ok(HttpResponse::Accepted().json(state.instances.status(id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/command",
	tag = "instances",
	request_body = CommandRequest,
	responses((status = 204, description = "Command sent to the server console"))
)]
#[post("/instances/{id}/command")]
pub async fn send_command(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<CommandRequest>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_SEND).await?;
	state.instances.send_command(id, &body.line).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/console",
	tag = "instances",
	responses((status = 200, description = "Most recent console lines", body = [String]))
)]
#[get("/instances/{id}/console")]
pub async fn console(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	query: web::Query<ConsoleQuery>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	let lines = query.lines.unwrap_or(DEFAULT_CONSOLE_LINES).min(MAX_CONSOLE_LINES);
	Ok(HttpResponse::Ok().json(state.instances.console_backlog(id, lines).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/stats",
	tag = "instances",
	responses((status = 200, description = "Latest resource and player statistics", body = obsidian_instances::InstanceStats))
)]
#[get("/instances/{id}/stats")]
pub async fn stats(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	Ok(HttpResponse::Ok().json(state.instances.stats(id).await?))
}
