use std::path::Path;

use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, delete, get, post, put, web};
use futures_util::TryStreamExt;
use obsidian_auth::{Permission, generate_token};
use obsidian_common::{CommonError, InstanceId};
use obsidian_content::{PublicBase, ResourcePack, ResourcePackList};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use utoipa::ToSchema;

use super::auth::support::{public_url, request_origin};
use super::content_targets::VersionRef;
use crate::auth::{AuthUser, require_instance_perm};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const MAX_PACK_BYTES: u64 = 512 * 1024 * 1024;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_resource_packs)
		.service(add_resource_pack)
		.service(upload_resource_pack)
		.service(reorder_resource_packs)
		.service(remove_resource_pack)
		.service(download_resource_pack);
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResourcePackResponse {
	pub packs: Vec<ResourcePack>,
	/// Direct download URL of the combined pack. Clients receive it through server.properties.
	pub url: Option<String>,
	/// False when players have to reach the pack through the browser origin that last edited the list.
	pub public_url_configured: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderRequest {
	/// Every pack id exactly once, highest priority first. The top pack wins when two packs share a file.
	pub ids: Vec<i64>,
}

async fn respond(state: &AppState, list: ResourcePackList) -> ApiResult<web::Json<ResourcePackResponse>> {
	Ok(web::Json(ResourcePackResponse {
		packs: list.packs,
		url: list.url,
		public_url_configured: public_url(state).await?.is_some(),
	}))
}

fn invalid(error: impl std::fmt::Display) -> ApiError {
	ApiError::from(CommonError::Invalid(error.to_string()))
}

async fn stream_pack(payload: &mut Multipart, target: &Path) -> ApiResult<String> {
	while let Some(mut field) = payload.try_next().await.map_err(invalid)? {
		if field.name() != Some("pack") {
			continue;
		}
		let name = field
			.content_disposition()
			.and_then(|disposition| disposition.get_filename())
			.and_then(|name| Path::new(name).file_name())
			.and_then(|name| name.to_str())
			.unwrap_or("resource-pack.zip")
			.to_owned();
		let mut file = tokio::fs::File::create(target).await.map_err(CommonError::from)?;
		let mut written: u64 = 0;
		while let Some(chunk) = field.try_next().await.map_err(invalid)? {
			written += chunk.len() as u64;
			if written > MAX_PACK_BYTES {
				return Err(invalid("the resource pack is larger than 512 MB"));
			}
			file.write_all(&chunk).await.map_err(CommonError::from)?;
		}
		file.flush().await.map_err(CommonError::from)?;
		return Ok(name);
	}
	Err(invalid("the multipart form needs a 'pack' field"))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/resource-packs",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Resource packs, highest priority first", body = ResourcePackResponse),
		(status = 403, description = "Missing files.read permission"),
	)
)]
#[get("/instances/{id}/resource-packs")]
pub async fn list_resource_packs(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<web::Json<ResourcePackResponse>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_READ).await?;
	let list = state.content.resource_packs(id).await?;
	respond(&state, list).await
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/resource-packs",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = VersionRef,
	responses(
		(status = 200, description = "Pack added at the top of the list", body = ResourcePackResponse),
		(status = 403, description = "Missing content.install permission"),
		(status = 409, description = "The pack must be downloaded manually"),
	)
)]
#[post("/instances/{id}/resource-packs")]
pub async fn add_resource_pack(
	state: web::Data<AppState>,
	user: AuthUser,
	req: HttpRequest,
	path: web::Path<InstanceId>,
	web::Json(request): web::Json<VersionRef>,
) -> ApiResult<web::Json<ResourcePackResponse>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	let configured = public_url(&state).await?;
	let origin = request_origin(&req);
	let base = PublicBase {
		configured: configured.as_deref(),
		origin: Some(&origin),
	};
	let list = state
		.content
		.add_platform_resource_pack(id, request.platform, &request.project_id, &request.version_id, base)
		.await?;
	respond(&state, list).await
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/resource-packs/upload",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	request_body(content_type = "multipart/form-data", description = "Form field 'pack' with a resource pack .zip (512 MB limit)"),
	responses(
		(status = 200, description = "Pack added at the top of the list", body = ResourcePackResponse),
		(status = 400, description = "Not a resource pack"),
		(status = 403, description = "Missing content.install permission"),
	)
)]
#[post("/instances/{id}/resource-packs/upload")]
pub async fn upload_resource_pack(
	state: web::Data<AppState>,
	user: AuthUser,
	req: HttpRequest,
	path: web::Path<InstanceId>,
	mut payload: Multipart,
) -> ApiResult<web::Json<ResourcePackResponse>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	let staged = state.paths.uploads_dir.join(format!("{}.part", generate_token()));
	let name = match stream_pack(&mut payload, &staged).await {
		Ok(name) => name,
		Err(error) => {
			let _ = tokio::fs::remove_file(&staged).await;
			return Err(error);
		}
	};
	let configured = public_url(&state).await?;
	let origin = request_origin(&req);
	let base = PublicBase {
		configured: configured.as_deref(),
		origin: Some(&origin),
	};
	let added = state.content.add_staged_resource_pack(id, &name, None, &staged, base).await;
	if added.is_err() {
		let _ = tokio::fs::remove_file(&staged).await;
	}
	respond(&state, added?).await
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/resource-packs/order",
	tag = "content",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = ReorderRequest,
	responses(
		(status = 200, description = "Packs reordered and the combined pack rebuilt", body = ResourcePackResponse),
		(status = 400, description = "The list does not match the current packs"),
		(status = 403, description = "Missing content.install permission"),
	)
)]
#[put("/instances/{id}/resource-packs/order")]
pub async fn reorder_resource_packs(
	state: web::Data<AppState>,
	user: AuthUser,
	req: HttpRequest,
	path: web::Path<InstanceId>,
	web::Json(request): web::Json<ReorderRequest>,
) -> ApiResult<web::Json<ResourcePackResponse>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_INSTALL).await?;
	let configured = public_url(&state).await?;
	let origin = request_origin(&req);
	let base = PublicBase {
		configured: configured.as_deref(),
		origin: Some(&origin),
	};
	let list = state.content.reorder_resource_packs(id, &request.ids, base).await?;
	respond(&state, list).await
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}/resource-packs/{pack_id}",
	tag = "content",
	params(
		("id" = i64, Path, description = "Instance id"),
		("pack_id" = i64, Path, description = "Resource pack id"),
	),
	responses(
		(status = 200, description = "Pack removed and the combined pack rebuilt", body = ResourcePackResponse),
		(status = 403, description = "Missing content.remove permission"),
		(status = 404, description = "No such pack"),
	)
)]
#[delete("/instances/{id}/resource-packs/{pack_id}")]
pub async fn remove_resource_pack(
	state: web::Data<AppState>,
	user: AuthUser,
	req: HttpRequest,
	path: web::Path<(InstanceId, i64)>,
) -> ApiResult<web::Json<ResourcePackResponse>> {
	let (id, pack_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONTENT_REMOVE).await?;
	let configured = public_url(&state).await?;
	let origin = request_origin(&req);
	let base = PublicBase {
		configured: configured.as_deref(),
		origin: Some(&origin),
	};
	let list = state.content.remove_resource_pack(id, pack_id, base).await?;
	respond(&state, list).await
}

#[utoipa::path(
	get,
	path = "/api/v1/resource-packs/{token}/pack.zip",
	tag = "content",
	params(("token" = String, Path, description = "Unguessable token from the pack URL")),
	responses(
		(status = 200, description = "Combined resource pack", content_type = "application/zip"),
		(status = 404, description = "No pack for this token"),
	)
)]
#[get("/resource-packs/{token}/pack.zip")]
pub async fn download_resource_pack(
	state: web::Data<AppState>,
	req: HttpRequest,
	path: web::Path<String>,
) -> ApiResult<HttpResponse> {
	let token = path.into_inner();
	let Some(file) = state.content.resource_pack_file(&token).await? else {
		return Err(CommonError::NotFound("resource pack".into()).into());
	};
	let file = NamedFile::open(file).map_err(CommonError::from)?;
	Ok(file.into_response(&req))
}
