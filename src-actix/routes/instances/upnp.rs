use actix_web::{HttpResponse, get, web};
use obsidian_auth::Permission;
use obsidian_common::InstanceId;

use super::dto::{MapUpnpInfo, UpnpInfo};
use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/upnp",
	tag = "instances",
	responses((status = 200, description = "UPnP port mapping state for the server port and its web maps", body = UpnpInfo))
)]
#[get("/instances/{id}/upnp")]
pub async fn upnp_status(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	let instance = state.instances.get(id).await?;
	let status = state.instances.upnp_status(id).await?;
	let map_enabled = state.instances.map_upnp_enabled(id).await?;
	let maps = state
		.instances
		.map_upnp_status(id)
		.await?
		.into_iter()
		.map(|(port, status)| MapUpnpInfo { port, status })
		.collect();
	Ok(HttpResponse::Ok().json(UpnpInfo {
		enabled: instance.upnp_enabled,
		status,
		map_enabled,
		maps,
	}))
}
