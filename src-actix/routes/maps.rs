
use std::path::Path;

use actix_web::{HttpResponse, get, post, put, web};
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};
use serde::Deserialize;
use utoipa::ToSchema;
use obsidian_maps::{
	ConfigEntry, DetectedMap, MapKind, MapSettingsUpdate, accept_download, apply_settings, bluemap_core_config, config_text, detect, port_path,
	scalar_entries, set_scalar_entry,
};

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_maps)
		.service(update_map)
		.service(set_map_upnp)
		.service(list_entries)
		.service(update_entry)
		.service(accept_bluemap_download);
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/maps",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Web maps found for this instance", body = Vec<DetectedMap>),
		(status = 403, description = "Missing console.read permission"),
	)
)]
#[get("/instances/{id}/maps")]
pub async fn list_maps(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<web::Json<Vec<DetectedMap>>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	Ok(web::Json(detect(&state.paths.instance_dir(id)).await?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/maps/{kind}",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("kind" = String, Path, description = "bluemap, dynmap, squaremap or pl3xmap"),
	),
	request_body = MapSettingsUpdate,
	responses(
		(status = 200, description = "Updated map settings; the map reads them on its next start", body = DetectedMap),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "The map cannot be edited here, or its port matches the server port"),
	)
)]
#[put("/instances/{id}/maps/{kind}")]
pub async fn update_map(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, MapKind)>,
	web::Json(update): web::Json<MapSettingsUpdate>,
) -> ApiResult<web::Json<DetectedMap>> {
	let (id, kind) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let instance = state.instances.get(id).await?;
	if update.port.is_some_and(|port| port == instance.port) {
		return Err(CommonError::Conflict("the map port cannot be the same as the server port".into()).into());
	}
	let dir = state.paths.instance_dir(id);
	let Some((relative, text)) = config_text(&dir, kind).await? else {
		return Err(CommonError::Conflict(format!("start the server once so {} can create its config", kind.label())).into());
	};
	let edited = apply_settings(kind, &text, &update)?;
	save_config(&dir, &relative, &edited).await?;
	let map = detect(&dir).await?.into_iter().find(|map| map.kind == kind);
	let map = map.ok_or_else(|| CommonError::NotFound(format!("{} map", kind.label())))?;
	Ok(web::Json(map))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EntryUpdate {
	/// Dotted path from the config root, as listed by the entries endpoint.
	pub path: String,
	pub value: String,
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/maps/{kind}/entries",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("kind" = String, Path, description = "bluemap, dynmap, squaremap or pl3xmap"),
	),
	responses(
		(status = 200, description = "Every scalar setting in the map config", body = Vec<ConfigEntry>),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "The config does not exist yet"),
	)
)]
#[get("/instances/{id}/maps/{kind}/entries")]
pub async fn list_entries(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, MapKind)>,
) -> ApiResult<web::Json<Vec<ConfigEntry>>> {
	let (id, kind) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let Some((_, text)) = config_text(&state.paths.instance_dir(id), kind).await? else {
		return Err(CommonError::Conflict(format!("start the server once so {} can create its config", kind.label())).into());
	};
	Ok(web::Json(scalar_entries(kind, &text)?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/maps/{kind}/entries",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("kind" = String, Path, description = "dynmap, squaremap or pl3xmap"),
	),
	request_body = EntryUpdate,
	responses(
		(status = 200, description = "Updated settings; the map reads them on its next start", body = Vec<ConfigEntry>),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "The value clashes with the server port or the config is read-only"),
	)
)]
#[put("/instances/{id}/maps/{kind}/entries")]
pub async fn update_entry(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, MapKind)>,
	web::Json(update): web::Json<EntryUpdate>,
) -> ApiResult<web::Json<Vec<ConfigEntry>>> {
	let (id, kind) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	if update.path == port_path(kind) {
		let port: u16 = update
			.value
			.trim()
			.parse()
			.map_err(|_| CommonError::Invalid(format!("{} is not a valid port", update.value)))?;
		if port == state.instances.get(id).await?.port {
			return Err(CommonError::Conflict("the map port cannot be the same as the server port".into()).into());
		}
	}
	let dir = state.paths.instance_dir(id);
	let Some((relative, text)) = config_text(&dir, kind).await? else {
		return Err(CommonError::Conflict(format!("start the server once so {} can create its config", kind.label())).into());
	};
	let edited = set_scalar_entry(kind, &text, &update.path, &update.value)?;
	save_config(&dir, &relative, &edited).await?;
	Ok(web::Json(scalar_entries(kind, &edited)?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/maps/bluemap/accept-download",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "BlueMap may download the Minecraft client file on its next start", body = DetectedMap),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "BlueMap has not written its core.conf yet, or the file has no accept-download setting"),
	)
)]
#[post("/instances/{id}/maps/bluemap/accept-download")]
pub async fn accept_bluemap_download(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<web::Json<DetectedMap>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let dir = state.paths.instance_dir(id);
	let Some((relative, text)) = bluemap_core_config(&dir).await? else {
		return Err(CommonError::Conflict("start the server once so BlueMap can create its config".into()).into());
	};
	save_config(&dir, &relative, &accept_download(&text)?).await?;
	let map = detect(&dir).await?.into_iter().find(|map| map.kind == MapKind::Bluemap);
	let map = map.ok_or_else(|| CommonError::NotFound("BlueMap map".into()))?;
	Ok(web::Json(map))
}

async fn save_config(dir: &Path, relative: &str, text: &str) -> ApiResult<()> {
	let path = dir.join(relative);
	let temp = path.with_extension("obsidian-tmp");
	tokio::fs::write(&temp, text).await.map_err(CommonError::from)?;
	tokio::fs::rename(&temp, &path).await.map_err(CommonError::from)?;
	Ok(())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MapUpnpUpdate {
	pub enabled: bool,
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/map-upnp",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = MapUpnpUpdate,
	responses(
		(status = 204, description = "Forwarding of the web map ports is updated"),
		(status = 403, description = "Missing settings.edit permission"),
	)
)]
#[put("/instances/{id}/map-upnp")]
pub async fn set_map_upnp(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	web::Json(update): web::Json<MapUpnpUpdate>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	state.instances.set_map_upnp(id, update.enabled).await?;
	Ok(HttpResponse::NoContent().finish())
}

