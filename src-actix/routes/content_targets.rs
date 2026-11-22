use actix_web::{HttpResponse, delete, get, post, put, web};
use obsidian_auth::Permission;
use obsidian_content::{InstalledWorld, Platform, ServerResourcePack};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(get_resource_pack)
		.service(set_resource_pack)
		.service(clear_resource_pack)
		.service(get_world)
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
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/resource-pack",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "The server resource pack, or null when none is set", body = Option<ServerResourcePack>),
		(status = 403, description = "Missing files.read permission"),
	)
)]
#[get("/instances/{id}/resource-pack")]
pub async fn get_resource_pack(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<web::Json<Option<ServerResourcePack>>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_READ).await?;
	Ok(web::Json(state.content.resource_pack(id).await?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/resource-pack",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = VersionRef,
	responses(
		(status = 200, description = "server.properties now points at this resource pack", body = ServerResourcePack),
		(status = 403, description = "Missing content.install permission"),
		(status = 409, description = "The pack must be downloaded manually"),
	)
)]
#[put("/instances/{id}/resource-pack")]
pub async fn set_resource_pack(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
	web::Json(request): web::Json<VersionRef>,
) -> ApiResult<web::Json<ServerResourcePack>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	Ok(web::Json(
		state
			.content
			.set_resource_pack(id, request.platform, &request.project_id, &request.version_id)
			.await?,
	))
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}/resource-pack",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 204, description = "Resource pack cleared"),
		(status = 403, description = "Missing content.remove permission"),
	)
)]
#[delete("/instances/{id}/resource-pack")]
pub async fn clear_resource_pack(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_REMOVE).await?;
	state.content.clear_resource_pack(id).await?;
	Ok(HttpResponse::NoContent().finish())
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
	Ok(web::Json(CurrentWorld {
		level_name: state.content.level_name(id).await?,
	}))
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
