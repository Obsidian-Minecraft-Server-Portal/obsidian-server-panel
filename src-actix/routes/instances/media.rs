use std::path::PathBuf;

use actix_multipart::Multipart;
use actix_web::{HttpResponse, delete, get, put, web};
use futures_util::TryStreamExt;
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const MAX_ICON_BYTES: usize = 32 * 1024 * 1024;

fn invalid(error: impl std::fmt::Display) -> ApiError {
	ApiError::from(CommonError::Invalid(error.to_string()))
}

async fn image_field(payload: &mut Multipart, name: &str) -> ApiResult<Vec<u8>> {
	while let Some(mut field) = payload.try_next().await.map_err(invalid)? {
		if field.name() != Some(name) {
			continue;
		}
		let mut bytes = Vec::new();
		while let Some(chunk) = field.try_next().await.map_err(invalid)? {
			if bytes.len() + chunk.len() > MAX_ICON_BYTES {
				return Err(invalid("the image is larger than 32 MB"));
			}
			bytes.extend_from_slice(&chunk);
		}
		return Ok(bytes);
	}
	Err(invalid(format!("the multipart form needs an '{name}' field")))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/icon",
	tag = "instances",
	request_body(content_type = "multipart/form-data", description = "Form field 'icon' with any image type"),
	responses((status = 204, description = "Icon resized to 64x64 and saved as server-icon.png"))
)]
#[put("/instances/{id}/icon")]
pub async fn upload_icon(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	mut payload: Multipart,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let bytes = image_field(&mut payload, "icon").await?;
	state.instances.set_icon(id, bytes).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/banner",
	tag = "instances",
	request_body(content_type = "multipart/form-data", description = "Form field 'banner' with any image type"),
	responses((status = 204, description = "Banner cover-cropped to 1280x360 and saved as JPEG"))
)]
#[put("/instances/{id}/banner")]
pub async fn upload_banner(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	mut payload: Multipart,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let bytes = image_field(&mut payload, "banner").await?;
	state.instances.set_banner(id, bytes).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}/banner",
	tag = "instances",
	responses((status = 204, description = "Banner removed; the generated landscape is shown instead"))
)]
#[delete("/instances/{id}/banner")]
pub async fn delete_banner(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	state.instances.clear_banner(id).await?;
	Ok(HttpResponse::NoContent().finish())
}

async fn serve_image(path: Option<PathBuf>, content_type: &'static str, id: InstanceId, kind: &str) -> ApiResult<HttpResponse> {
	let missing = || ApiError::from(CommonError::NotFound(format!("{kind} for instance {id}")));
	let path = path.ok_or_else(missing)?;
	match tokio::fs::read(path).await {
		Ok(bytes) => Ok(HttpResponse::Ok().content_type(content_type).body(bytes)),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(missing()),
		Err(error) => Err(CommonError::from(error).into()),
	}
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/icon",
	tag = "instances",
	responses(
		(status = 200, description = "Server icon as PNG", content_type = "image/png"),
		(status = 404, description = "No icon set"),
	)
)]
#[get("/instances/{id}/icon")]
pub async fn get_icon(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	let instance = state.instances.get(id).await?;
	serve_image(instance.icon_path, "image/png", id, "icon").await
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/banner",
	tag = "instances",
	responses(
		(status = 200, description = "Server banner as JPEG", content_type = "image/jpeg"),
		(status = 404, description = "No banner set"),
	)
)]
#[get("/instances/{id}/banner")]
pub async fn get_banner(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	let instance = state.instances.get(id).await?;
	serve_image(instance.banner_path, "image/jpeg", id, "banner").await
}
