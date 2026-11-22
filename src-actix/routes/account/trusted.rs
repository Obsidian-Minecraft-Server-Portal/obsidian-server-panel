use actix_web::{HttpResponse, web};
use obsidian_auth::TrustedIp;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

#[utoipa::path(
	get,
	path = "/api/v1/account/trusted-ips",
	tag = "account",
	responses((status = 200, description = "Addresses that sign in without a second factor", body = Vec<TrustedIp>))
)]
pub async fn list_trusted_ips(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.list_trusted_ips(user.id).await?))
}

#[utoipa::path(
	delete,
	path = "/api/v1/account/trusted-ips/{id}",
	tag = "account",
	params(("id" = i64, Path, description = "Trusted address id")),
	responses((status = 204, description = "Address revoked and its sessions signed out"))
)]
pub async fn revoke_trusted_ip(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<i64>,
) -> ApiResult<HttpResponse> {
	state.auth.revoke_trusted_ip(user.id, id.into_inner()).await?;
	Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("/account/trusted-ips").route(web::get().to(list_trusted_ips)))
		.service(web::resource("/account/trusted-ips/{id}").route(web::delete().to(revoke_trusted_ip)));
}
