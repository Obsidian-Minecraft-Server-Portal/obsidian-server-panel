use actix_web::{HttpResponse, get, post, web};
use obsidian_auth::Permission;
use obsidian_content::{ContentSource, InstalledWorld, Platform};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(get_world)
		.service(install_world);
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VersionRef {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub project_id: String,
	pub version_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentWorld {
	pub level_name: String,
	pub source: Option<ContentSource>,
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/world",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "The world folder the server loads", body = CurrentWorld),
		(status = 403, description = "Missing files.read permission"),
	)
)]
#[get("/instances/{id}/world")]
pub async fn get_world(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<web::Json<CurrentWorld>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_READ).await?;
	let level_name = state.content.level_name(id).await?;
	let source = state.content.world_source(id, &level_name).await?;
	Ok(web::Json(CurrentWorld { level_name, source }))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/world",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = VersionRef,
	responses(
		(status = 201, description = "World extracted into a new folder and selected as level-name", body = InstalledWorld),
		(status = 400, description = "The download is not a Minecraft world"),
		(status = 403, description = "Missing content.install or files.write permission"),
		(status = 409, description = "The server is running"),
	)
)]
#[post("/instances/{id}/world")]
pub async fn install_world(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
	web::Json(request): web::Json<VersionRef>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL | Permission::FILES_WRITE).await?;
	let world = state
		.content
		.install_world(
			&state.instances,
			id,
			request.platform,
			&request.project_id,
			&request.version_id,
			Some(user.id),
		)
		.await?;
	Ok(HttpResponse::Created().json(world))
}
