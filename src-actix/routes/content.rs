use actix_multipart::Multipart;
use actix_web::{HttpResponse, delete, get, post, web};
use futures_util::TryStreamExt;
use obsidian_auth::Permission;
use obsidian_common::{CommonError, Loader};
use obsidian_content::{
	ContentUpdate, Facets, InstallRequest, InstalledContent, Kind, Platform, Project, ProjectVersion, SearchPage,
	SearchQuery, Side, VersionFilter, icon_mime,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;
const MAX_UPLOAD_BYTES: usize = 512 << 20;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(platforms)
		.service(search)
		.service(facets)
		.service(project)
		.service(project_versions)
		.service(list_content)
		.service(install_content)
		.service(content_updates)
		.service(update_all_content)
		.service(update_content)
		.service(upload_content)
		.service(remove_content)
		.service(toggle_content)
		.service(content_icon);
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlatformInfo {
	pub id: u8,
	pub name: &'static str,
	pub available: bool,
	pub kinds: Vec<Kind>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
	platform: String,
	kind: Option<Kind>,
	#[serde(default)]
	query: String,
	loader: Option<Loader>,
	mc_version: Option<String>,
	categories: Option<String>,
	excluded_categories: Option<String>,
	environments: Option<String>,
	open_source: Option<bool>,
	#[serde(default)]
	page: u32,
	limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct FacetsParams {
	kind: Option<Kind>,
}

#[derive(Debug, Deserialize)]
pub struct VersionParams {
	loader: Option<Loader>,
	mc_version: Option<String>,
	kind: Option<Kind>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ToggleRequest {
	pub enabled: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRequest {
	pub version_id: String,
}

#[utoipa::path(
	get,
	path = "/api/v1/content/platforms",
	tag = "content",
	responses(
		(status = 200, description = "Content platforms and whether this build can reach them", body = Vec<PlatformInfo>),
		(status = 401, description = "Authentication required"),
	)
)]
#[get("/content/platforms")]
pub async fn platforms(state: web::Data<AppState>, _user: AuthUser) -> web::Json<Vec<PlatformInfo>> {
	web::Json(
		state
			.content
			.platforms()
			.iter()
			.map(|platform| PlatformInfo {
				id: *platform as u8,
				name: platform.name(),
				available: state.content.is_available(*platform),
				kinds: state.content.kinds(*platform),
			})
			.collect(),
	)
}

#[utoipa::path(
	get,
	path = "/api/v1/content/search",
	tag = "content",
	params(
		("platform" = String, Query, description = "Platform name or numeric id"),
		("kind" = Option<String>, Query, description = "mod, plugin, modpack, datapack, resourcepack or world"),
		("query" = Option<String>, Query, description = "Free-text search"),
		("loader" = Option<String>, Query, description = "Mod loader filter"),
		("mc_version" = Option<String>, Query, description = "Minecraft version filter"),
		("page" = Option<u32>, Query, description = "Zero-based page"),
		("limit" = Option<u32>, Query, description = "Page size, max 50"),
	),
	responses(
		(status = 200, description = "Matching projects", body = SearchPage),
		(status = 400, description = "Unknown platform"),
		(status = 501, description = "Platform is not configured in this build"),
	)
)]
#[get("/content/search")]
pub async fn search(
	state: web::Data<AppState>,
	_user: AuthUser,
	web::Query(params): web::Query<SearchParams>,
) -> ApiResult<web::Json<SearchPage>> {
	let platform: Platform = params.platform.parse()?;
	let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
	let query = SearchQuery {
		text: params.query,
		kind: params.kind.unwrap_or(Kind::Mod),
		mc_version: params.mc_version,
		loader: params.loader,
		categories: split_list(params.categories),
		excluded_categories: split_list(params.excluded_categories),
		environments: parse_sides(params.environments.as_deref())?,
		open_source: params.open_source.unwrap_or(false),
		offset: params.page.saturating_mul(limit),
		limit,
	};
	Ok(web::Json(state.content.search(platform, &query).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/content/{platform}/facets",
	tag = "content",
	params(
		("platform" = String, Path, description = "Platform name or numeric id"),
		("kind" = Option<String>, Query, description = "mod, plugin, modpack, datapack, resourcepack or world"),
	),
	responses(
		(status = 200, description = "Filters the platform offers for this kind", body = Facets),
		(status = 400, description = "Unknown platform"),
	)
)]
#[get("/content/{platform}/facets")]
pub async fn facets(
	state: web::Data<AppState>,
	_user: AuthUser,
	path: web::Path<String>,
	web::Query(params): web::Query<FacetsParams>,
) -> ApiResult<web::Json<Facets>> {
	let platform: Platform = path.parse()?;
	Ok(web::Json(state.content.facets(platform, params.kind.unwrap_or(Kind::Mod)).await?))
}

fn split_list(value: Option<String>) -> Vec<String> {
	value
		.unwrap_or_default()
		.split(',')
		.map(str::trim)
		.filter(|item| !item.is_empty())
		.map(str::to_owned)
		.collect()
}

fn parse_sides(value: Option<&str>) -> ApiResult<Vec<Side>> {
	value
		.unwrap_or_default()
		.split(',')
		.map(str::trim)
		.filter(|item| !item.is_empty())
		.map(|item| match item {
			"client" => Ok(Side::Client),
			"server" => Ok(Side::Server),
			other => Err(CommonError::Invalid(format!("unknown side '{other}'")).into()),
		})
		.collect()
}

#[utoipa::path(
	get,
	path = "/api/v1/content/{platform}/projects/{id}",
	tag = "content",
	params(
		("platform" = String, Path, description = "Platform name or numeric id"),
		("id" = String, Path, description = "Project id on the platform"),
	),
	responses(
		(status = 200, description = "Project details", body = Project),
		(status = 404, description = "Project not found"),
	)
)]
#[get("/content/{platform}/projects/{id}")]
pub async fn project(
	state: web::Data<AppState>,
	_user: AuthUser,
	path: web::Path<(String, String)>,
) -> ApiResult<web::Json<Project>> {
	let (platform, id) = path.into_inner();
	let platform: Platform = platform.parse()?;
	Ok(web::Json(state.content.project(platform, &id).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/content/{platform}/projects/{id}/versions",
	tag = "content",
	params(
		("platform" = String, Path, description = "Platform name or numeric id"),
		("id" = String, Path, description = "Project id on the platform"),
		("loader" = Option<String>, Query, description = "Mod loader filter"),
		("mc_version" = Option<String>, Query, description = "Minecraft version filter"),
		("kind" = Option<String>, Query, description = "Content kind, narrows files such as datapack zips"),
	),
	responses(
		(status = 200, description = "Versions, newest first where the platform reports order", body = Vec<ProjectVersion>),
		(status = 404, description = "Project not found"),
	)
)]
#[get("/content/{platform}/projects/{id}/versions")]
pub async fn project_versions(
	state: web::Data<AppState>,
	_user: AuthUser,
	path: web::Path<(String, String)>,
	web::Query(params): web::Query<VersionParams>,
) -> ApiResult<web::Json<Vec<ProjectVersion>>> {
	let (platform, id) = path.into_inner();
	let platform: Platform = platform.parse()?;
	let filter = VersionFilter {
		mc_version: params.mc_version,
		loader: params.loader,
		kind: params.kind,
	};
	Ok(web::Json(state.content.versions(platform, &id, &filter).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/content",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Installed mods and plugins", body = Vec<InstalledContent>),
		(status = 403, description = "Missing files.read permission"),
	)
)]
#[get("/instances/{id}/content")]
pub async fn list_content(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<web::Json<Vec<InstalledContent>>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_READ).await?;
	Ok(web::Json(state.content.list(id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/content",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = InstallRequest,
	responses(
		(status = 201, description = "Installed content", body = InstalledContent),
		(status = 403, description = "Missing content.install permission"),
		(status = 409, description = "Already installed or must be downloaded manually"),
	)
)]
#[post("/instances/{id}/content")]
pub async fn install_content(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
	web::Json(request): web::Json<InstallRequest>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	let installed = state
		.content
		.install(
			id,
			request.platform,
			&request.project_id,
			&request.version_id,
			request.kind.unwrap_or(Kind::Mod),
			Some(user.id),
		)
		.await?;
	Ok(HttpResponse::Created().json(installed))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/content/updates",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Installed items with a newer version available", body = Vec<ContentUpdate>),
		(status = 403, description = "Missing files.read permission"),
	)
)]
#[get("/instances/{id}/content/updates")]
pub async fn content_updates(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<web::Json<Vec<ContentUpdate>>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_READ).await?;
	Ok(web::Json(state.content.updates(id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/content/updates",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Every item that was updated to its latest compatible version", body = Vec<InstalledContent>),
		(status = 403, description = "Missing content.install permission"),
	)
)]
#[post("/instances/{id}/content/updates")]
pub async fn update_all_content(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<web::Json<Vec<InstalledContent>>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	Ok(web::Json(state.content.update_all(id, Some(user.id)).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/content/{mod_id}/update",
	tag = "content",
	params(
		("id" = i64, Path, description = "Instance id"),
		("mod_id" = i64, Path, description = "Installed content id"),
	),
	request_body = UpdateRequest,
	responses(
		(status = 200, description = "Updated content", body = InstalledContent),
		(status = 400, description = "Content is not linked to a platform"),
		(status = 403, description = "Missing content.install permission"),
		(status = 404, description = "Not installed"),
	)
)]
#[post("/instances/{id}/content/{mod_id}/update")]
pub async fn update_content(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(i64, i64)>,
	web::Json(request): web::Json<UpdateRequest>,
) -> ApiResult<web::Json<InstalledContent>> {
	let (id, mod_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	Ok(web::Json(
		state
			.content
			.apply_update(id, mod_id, &request.version_id, Some(user.id))
			.await?,
	))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/content/upload",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	request_body(content = String, content_type = "multipart/form-data"),
	responses(
		(status = 200, description = "Uploaded jar", body = InstalledContent),
		(status = 400, description = "Missing file or not a jar"),
		(status = 403, description = "Missing content.install permission"),
	)
)]
#[post("/instances/{id}/content/upload")]
pub async fn upload_content(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
	mut payload: Multipart,
) -> ApiResult<web::Json<InstalledContent>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	let mut upload = None;
	while let Some(mut field) = payload.try_next().await.map_err(invalid_upload)? {
		let Some(name) = field
			.content_disposition()
			.and_then(|disposition| disposition.get_filename())
			.map(str::to_owned)
		else {
			continue;
		};
		let mut bytes = Vec::new();
		while let Some(chunk) = field.try_next().await.map_err(invalid_upload)? {
			if bytes.len() + chunk.len() > MAX_UPLOAD_BYTES {
				return Err(CommonError::Invalid("file exceeds the upload limit".into()).into());
			}
			bytes.extend_from_slice(&chunk);
		}
		upload = Some((name, bytes));
		break;
	}
	let (name, bytes) = upload.ok_or_else(|| CommonError::Invalid("upload contains no file".into()))?;
	Ok(web::Json(state.content.upload(id, &name, bytes).await?))
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}/content/{mod_id}",
	tag = "content",
	params(
		("id" = i64, Path, description = "Instance id"),
		("mod_id" = i64, Path, description = "Installed content id"),
	),
	responses(
		(status = 204, description = "Removed"),
		(status = 403, description = "Missing content.remove permission"),
		(status = 404, description = "Not installed"),
	)
)]
#[delete("/instances/{id}/content/{mod_id}")]
pub async fn remove_content(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(i64, i64)>,
) -> ApiResult<HttpResponse> {
	let (id, mod_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_REMOVE).await?;
	let item = state.content.get(id, mod_id).await?;
	state.content.remove(id, &item.filename).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/content/{mod_id}/toggle",
	tag = "content",
	params(
		("id" = i64, Path, description = "Instance id"),
		("mod_id" = i64, Path, description = "Installed content id"),
	),
	request_body = ToggleRequest,
	responses(
		(status = 200, description = "Updated content", body = InstalledContent),
		(status = 403, description = "Missing content.install permission"),
		(status = 404, description = "Not installed"),
	)
)]
#[post("/instances/{id}/content/{mod_id}/toggle")]
pub async fn toggle_content(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(i64, i64)>,
	web::Json(request): web::Json<ToggleRequest>,
) -> ApiResult<web::Json<InstalledContent>> {
	let (id, mod_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	let item = state.content.get(id, mod_id).await?;
	Ok(web::Json(
		state.content.toggle(id, &item.filename, request.enabled).await?,
	))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/content/{mod_id}/icon",
	tag = "content",
	params(
		("id" = i64, Path, description = "Instance id"),
		("mod_id" = i64, Path, description = "Installed content id"),
	),
	responses(
		(status = 200, description = "Icon image bytes"),
		(status = 404, description = "No icon stored"),
	)
)]
#[get("/instances/{id}/content/{mod_id}/icon")]
pub async fn content_icon(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(i64, i64)>,
) -> ApiResult<HttpResponse> {
	let (id, mod_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_READ).await?;
	let bytes = state
		.content
		.icon(id, mod_id)
		.await?
		.ok_or_else(|| CommonError::NotFound("icon".into()))?;
	Ok(HttpResponse::Ok().content_type(icon_mime(&bytes)).body(bytes))
}

fn invalid_upload(error: actix_multipart::MultipartError) -> CommonError {
	CommonError::Invalid(format!("malformed upload: {error}"))
}
