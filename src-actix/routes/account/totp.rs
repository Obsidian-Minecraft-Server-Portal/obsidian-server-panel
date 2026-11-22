use actix_web::{HttpResponse, web};
use obsidian_auth::TotpSetup;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CodeRequest {
	pub code: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasswordRequest {
	pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecoveryCodes {
	/// Each code works once. Shown only in this response.
	pub recovery_codes: Vec<String>,
}

#[utoipa::path(
	post,
	path = "/api/v1/account/totp/setup",
	tag = "account",
	responses((status = 200, description = "Pending authenticator secret and QR code", body = TotpSetup))
)]
pub async fn totp_setup(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.totp_setup(&user).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/account/totp/confirm",
	tag = "account",
	request_body = CodeRequest,
	responses(
		(status = 200, description = "Two-factor enabled", body = RecoveryCodes),
		(status = 400, description = "Code is invalid")
	)
)]
pub async fn totp_confirm(
	state: web::Data<AppState>,
	user: AuthUser,
	body: web::Json<CodeRequest>,
) -> ApiResult<HttpResponse> {
	let recovery_codes = state.auth.totp_confirm(&user, &body.code).await?;
	Ok(HttpResponse::Ok().json(RecoveryCodes { recovery_codes }))
}

#[utoipa::path(
	post,
	path = "/api/v1/account/totp/disable",
	tag = "account",
	request_body = PasswordRequest,
	responses((status = 204, description = "Two-factor disabled"), (status = 400, description = "Password is incorrect"))
)]
pub async fn totp_disable(
	state: web::Data<AppState>,
	user: AuthUser,
	body: web::Json<PasswordRequest>,
) -> ApiResult<HttpResponse> {
	state.auth.totp_disable(&user, &body.password).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/account/totp/recovery-codes",
	tag = "account",
	request_body = CodeRequest,
	responses((status = 200, description = "Fresh recovery codes; older codes stop working", body = RecoveryCodes))
)]
pub async fn totp_regenerate(
	state: web::Data<AppState>,
	user: AuthUser,
	body: web::Json<CodeRequest>,
) -> ApiResult<HttpResponse> {
	let recovery_codes = state.auth.regenerate_recovery_codes(&user, &body.code).await?;
	Ok(HttpResponse::Ok().json(RecoveryCodes { recovery_codes }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("/account/totp/setup").route(web::post().to(totp_setup)))
		.service(web::resource("/account/totp/confirm").route(web::post().to(totp_confirm)))
		.service(web::resource("/account/totp/disable").route(web::post().to(totp_disable)))
		.service(web::resource("/account/totp/recovery-codes").route(web::post().to(totp_regenerate)));
}
