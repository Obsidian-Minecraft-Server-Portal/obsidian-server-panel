use actix_multipart::Multipart;
use actix_web::{HttpResponse, get, put, web};
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

async fn icon_field(payload: &mut Multipart) -> ApiResult<Vec<u8>> {
	while let Some(mut field) = payload.try_next().await.map_err(invalid)? {
		if field.name() != Some("icon") {
			continue;
		}
		let mut bytes = Vec::new();
		while let Some(chunk) = field.try_next().await.map_err(invalid)? {
			if bytes.len() + chunk.len() > MAX_ICON_BYTES {
				return Err(invalid("the icon image is larger than 32 MB"));
			}
			bytes.extend_from_slice(&chunk);
		}
		return Ok(bytes);
	}
	Err(invalid("the multipart form needs an 'icon' field"))
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
	let bytes = icon_field(&mut payload).await?;
	state.instances.set_icon(id, bytes).await?;
	Ok(HttpResponse::NoContent().finish())
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
	let missing = || ApiError::from(CommonError::NotFound(format!("icon for instance {id}")));
	let icon = instance.icon_path.ok_or_else(missing)?;
	match tokio::fs::read(icon).await {
		Ok(bytes) => Ok(HttpResponse::Ok().content_type("image/png").body(bytes)),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(missing()),
		Err(error) => Err(CommonError::from(error).into()),
	}
}
