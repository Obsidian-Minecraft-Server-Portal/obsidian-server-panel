use actix_web::{HttpResponse, get, post, web};
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};
use obsidian_instances::PlayerAction;
use serde_json::Value;

use super::dto::PlayerNameRequest;
use crate::auth::{AuthUser, require_instance_perm};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/players",
	tag = "instances",
	responses((status = 200, description = "Known players with online, op, whitelist and ban state", body = [obsidian_instances::Player]))
)]
#[get("/instances/{id}/players")]
pub async fn list_players(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	Ok(HttpResponse::Ok().json(state.instances.players(id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/players/{action}",
	tag = "instances",
	request_body = PlayerNameRequest,
	params(("action" = PlayerAction, Path, description = "op, deop, whitelist, unwhitelist, kick, ban or pardon")),
	responses((status = 204, description = "Player action applied"))
)]
#[post("/instances/{id}/players/{action}")]
pub async fn player_action(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
	body: web::Json<PlayerNameRequest>,
) -> ApiResult<HttpResponse> {
	let (id, action) = path.into_inner();
	let action = serde_json::from_value::<PlayerAction>(Value::String(action))
		.map_err(|_| ApiError::from(CommonError::Invalid("unknown player action".into())))?;
	require_instance_perm(&state, &user, id, Permission::PLAYERS_MANAGE).await?;
	state.instances.player_action(id, &body.name, action).await?;
	Ok(HttpResponse::NoContent().finish())
}
