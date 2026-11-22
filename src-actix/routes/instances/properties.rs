use std::collections::BTreeMap;

use actix_web::{HttpResponse, get, put, web};
use obsidian_auth::Permission;
use obsidian_common::InstanceId;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/properties",
	tag = "instances",
	responses((status = 200, description = "server.properties as key/value pairs", body = BTreeMap<String, String>))
)]
#[get("/instances/{id}/properties")]
pub async fn get_properties(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_READ).await?;
	Ok(HttpResponse::Ok().json(state.instances.properties(id).await?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/properties",
	tag = "instances",
	request_body = BTreeMap<String, String>,
	responses((status = 204, description = "server.properties saved; applies on next start"))
)]
#[put("/instances/{id}/properties")]
pub async fn set_properties(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<BTreeMap<String, String>>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	state.instances.set_properties(id, body.into_inner()).await?;
	Ok(HttpResponse::NoContent().finish())
}
