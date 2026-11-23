mod access;
pub mod handlers;
pub mod model;
mod store;

use actix_web::{get, web};
use obsidian_auth::{PERMISSION_NAMES, Role};

use crate::auth::AuthUser;
use crate::error::ApiResult;
use model::{PermissionCatalog, RolePreset, permission_names};

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(permissions)
		.service(handlers::list_members)
		.service(handlers::member_candidates)
		.service(handlers::add_member)
		.service(handlers::update_member)
		.service(handlers::remove_member);
}

#[utoipa::path(
	get,
	path = "/api/v1/permissions",
	tag = "members",
	responses(
		(status = 200, description = "Permission names and role presets", body = PermissionCatalog),
		(status = 401, description = "Authentication required"),
	)
)]
#[get("/permissions")]
pub async fn permissions(_user: AuthUser) -> ApiResult<web::Json<PermissionCatalog>> {
	let catalog = PermissionCatalog {
		permissions: PERMISSION_NAMES.iter().map(|(name, _)| (*name).to_owned()).collect(),
		roles: Role::ALL
			.into_iter()
			.map(|role| RolePreset {
				role: role.as_str().to_owned(),
				permissions: permission_names(role.permissions()),
			})
			.collect(),
	};
	Ok(web::Json(catalog))
}
