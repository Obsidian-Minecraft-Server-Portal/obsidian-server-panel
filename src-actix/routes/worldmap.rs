use actix_files::NamedFile;
use actix_web::{HttpResponse, get, post, put, web};
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};
use obsidian_worldmap_service::{WorldMapManifest, WorldMapPlayers, WorldMapSettings, WorldMapStatus};

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(get_settings)
		.service(update_settings)
		.service(render)
		.service(status)
		.service(manifest)
		.service(file)
		.service(players)
		.service(head);
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/worldmap/settings",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "World map settings for this instance", body = WorldMapSettings),
		(status = 403, description = "Missing settings.edit permission"),
	)
)]
#[get("/instances/{id}/worldmap/settings")]
pub async fn get_settings(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<web::Json<WorldMapSettings>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	Ok(web::Json(state.worldmap.settings(id).await?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/worldmap/settings",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = WorldMapSettings,
	responses(
		(status = 200, description = "Saved settings. Turning the map on starts its first render", body = WorldMapSettings),
		(status = 400, description = "A value is out of range"),
		(status = 403, description = "Missing settings.edit permission"),
	)
)]
#[put("/instances/{id}/worldmap/settings")]
pub async fn update_settings(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	web::Json(update): web::Json<WorldMapSettings>,
) -> ApiResult<web::Json<WorldMapSettings>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	Ok(web::Json(state.worldmap.update_settings(id, update).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/worldmap/render",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 202, description = "A render is queued. Progress appears on the tasks topic"),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "The world map is not enabled"),
	)
)]
#[post("/instances/{id}/worldmap/render")]
pub async fn render(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	state.worldmap.render(id).await?;
	Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/worldmap/status",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Render state of the world map", body = WorldMapStatus),
		(status = 403, description = "Missing map.view permission"),
	)
)]
#[get("/instances/{id}/worldmap/status")]
pub async fn status(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<web::Json<WorldMapStatus>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_VIEW).await?;
	Ok(web::Json(state.worldmap.status(id).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/worldmap/dimensions/{folder}",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("folder" = String, Path, description = "Dimension folder from the status response"),
	),
	responses(
		(status = 200, description = "Sections, tiles and atlas of one dimension", body = WorldMapManifest),
		(status = 403, description = "Missing map.view permission"),
		(status = 404, description = "The dimension has no render yet"),
	)
)]
#[get("/instances/{id}/worldmap/dimensions/{folder}")]
pub async fn manifest(state: web::Data<AppState>, user: AuthUser, path: web::Path<(InstanceId, String)>) -> ApiResult<web::Json<WorldMapManifest>> {
	let (id, folder) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_VIEW).await?;
	Ok(web::Json(state.worldmap.manifest(id, &folder).await?))
}

#[get("/instances/{id}/worldmap/files/{path:.*}")]
pub async fn file(state: web::Data<AppState>, user: AuthUser, path: web::Path<(InstanceId, String)>) -> ApiResult<NamedFile> {
	let (id, relative) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_VIEW).await?;
	let location = state.worldmap.file(id, &relative).await?;
	NamedFile::open(location).map_err(|_| CommonError::NotFound(format!("map file {relative}")).into())
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/worldmap/players",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "The latest reading of who is online and where", body = WorldMapPlayers),
		(status = 403, description = "Missing map.players permission"),
	)
)]
#[get("/instances/{id}/worldmap/players")]
pub async fn players(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<web::Json<WorldMapPlayers>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_PLAYERS).await?;
	Ok(web::Json(state.worldmap.players(id).await?))
}

#[get("/instances/{id}/worldmap/heads/{file}")]
pub async fn head(state: web::Data<AppState>, user: AuthUser, path: web::Path<(InstanceId, String)>) -> ApiResult<NamedFile> {
	let (id, name) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_PLAYERS).await?;
	let location = state
		.worldmap
		.head_path(&name)
		.ok_or_else(|| CommonError::NotFound(format!("head {name}")))?;
	NamedFile::open(location).map_err(|_| CommonError::NotFound(format!("head {name}")).into())
}
