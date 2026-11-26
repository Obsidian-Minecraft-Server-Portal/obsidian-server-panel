use actix_files::NamedFile;
use actix_web::http::header::{self, HeaderValue};
use actix_web::middleware::Compress;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, put, web};
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};
use obsidian_worldmap_service::{WorldMapFocus, WorldMapManifest, WorldMapPlayers, WorldMapSettings, WorldMapStatus};
use serde::Deserialize;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(get_settings)
		.service(update_settings)
		.service(render)
		.service(focus)
		.service(status)
		.service(manifest)
		.service(file)
		.service(players)
		.service(skin)
		.service(detail)
		.service(
			web::resource("/instances/{id}/worldmap/dimensions/{folder}/structure/cells/{cx}/{cy}/{cz}")
				.wrap(Compress::default())
				.route(web::get().to(structure_cell)),
		);
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
	path = "/api/v1/instances/{id}/worldmap/focus",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = WorldMapFocus,
	responses(
		(status = 202, description = "The view is recorded, and its regions are drawn first"),
		(status = 403, description = "Missing map.view permission"),
	)
)]
#[post("/instances/{id}/worldmap/focus")]
pub async fn focus(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>, view: web::Json<WorldMapFocus>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_VIEW).await?;
	state.worldmap.focus(id, view.into_inner()).await?;
	Ok(HttpResponse::Accepted().finish())
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
pub async fn file(
	state: web::Data<AppState>,
	user: AuthUser,
	req: HttpRequest,
	path: web::Path<(InstanceId, String)>,
	query: web::Query<VersionQuery>,
) -> ApiResult<HttpResponse> {
	let (id, relative) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_VIEW).await?;
	let location = state.worldmap.file(id, &relative).await?;
	let file = NamedFile::open(location).map_err(|_| CommonError::NotFound(format!("map file {relative}")))?;
	let mut response = file.respond_to(&req);
	response
		.headers_mut()
		.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache_control(query.v.is_some())));
	Ok(response)
}

/// The structure of one cell: its sections and the border around them, as one bundle, compressed when the client accepts it.
/// A `v` query marks the copy as immutable.
pub async fn structure_cell(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String, i32, i32, i32)>,
	query: web::Query<VersionQuery>,
) -> ApiResult<HttpResponse> {
	let (id, folder, cx, cy, cz) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_VIEW).await?;
	let bundle = state.worldmap.structure_cell(id, &folder, (cx, cy, cz)).await?;
	Ok(HttpResponse::Ok()
		.content_type("application/octet-stream")
		.insert_header(("Cache-Control", cache_control(query.v.is_some())))
		.body(bundle))
}

/// One region's textured tile at `tx`, `tz` within it, drawn on first request. A `v` query marks the copy as immutable.
#[get("/instances/{id}/worldmap/dimensions/{folder}/detail/{rx}/{rz}/{tx}/{tz}")]
pub async fn detail(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String, i32, i32, usize, usize)>,
	query: web::Query<VersionQuery>,
) -> ApiResult<HttpResponse> {
	let (id, folder, rx, rz, tx, tz) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_VIEW).await?;
	let location = state.worldmap.detail(id, &folder, (rx, rz), (tx, tz)).await?;
	let png = tokio::fs::read(&location).await.map_err(|error| CommonError::Other(error.to_string()))?;
	Ok(HttpResponse::Ok()
		.content_type("image/png")
		.insert_header(("Cache-Control", cache_control(query.v.is_some())))
		.body(png))
}

#[derive(Deserialize)]
pub struct VersionQuery {
	v: Option<String>,
}

fn cache_control(versioned: bool) -> &'static str {
	if versioned { "public, max-age=31536000, immutable" } else { "no-cache" }
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

#[get("/instances/{id}/worldmap/skins/{file}")]
pub async fn skin(state: web::Data<AppState>, user: AuthUser, path: web::Path<(InstanceId, String)>) -> ApiResult<NamedFile> {
	let (id, name) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::MAP_PLAYERS).await?;
	let location = state
		.worldmap
		.skin_path(&name)
		.ok_or_else(|| CommonError::NotFound(format!("skin {name}")))?;
	NamedFile::open(location).map_err(|_| CommonError::NotFound(format!("skin {name}")).into())
}
