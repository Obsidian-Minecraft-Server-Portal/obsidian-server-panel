pub mod tokens;
pub mod totp;
pub mod trusted;

use actix_web::{HttpRequest, HttpResponse, web};
use obsidian_auth::{SecurityEvent, UserProfile};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::routes::auth::support::{client, session_token};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasswordChangeRequest {
	pub current_password: String,
	pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EmailChangeRequest {
	pub password: String,
	/// Empty or null clears the address.
	pub email: Option<String>,
}

#[utoipa::path(
	get,
	path = "/api/v1/account",
	tag = "account",
	responses((status = 200, description = "Account profile", body = UserProfile))
)]
pub async fn profile(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.profile(user.id).await?))
}

#[utoipa::path(
	patch,
	path = "/api/v1/account",
	tag = "account",
	request_body = EmailChangeRequest,
	responses(
		(status = 200, description = "Updated profile", body = UserProfile),
		(status = 400, description = "Password is incorrect or email is invalid")
	)
)]
pub async fn update_email(
	state: web::Data<AppState>,
	user: AuthUser,
	body: web::Json<EmailChangeRequest>,
) -> ApiResult<HttpResponse> {
	let email = body.email.as_deref().filter(|email| !email.is_empty());
	state.auth.update_email(user.id, &body.password, email).await?;
	Ok(HttpResponse::Ok().json(state.auth.profile(user.id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/account/password",
	tag = "account",
	request_body = PasswordChangeRequest,
	responses(
		(status = 204, description = "Password changed; other sessions were signed out"),
		(status = 400, description = "Current password is incorrect")
	)
)]
pub async fn change_password(
	state: web::Data<AppState>,
	req: HttpRequest,
	user: AuthUser,
	body: web::Json<PasswordChangeRequest>,
) -> ApiResult<HttpResponse> {
	let keep = session_token(&req);
	state
		.auth
		.change_password(
			user.id,
			&body.current_password,
			&body.new_password,
			keep.as_deref(),
			&client(&req),
		)
		.await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	get,
	path = "/api/v1/account/security-events",
	tag = "account",
	responses((status = 200, description = "Recent security events", body = Vec<SecurityEvent>))
)]
pub async fn security_events(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.security_events(user.id).await?))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/account")
			.route(web::get().to(profile))
			.route(web::patch().to(update_email)),
	)
	.service(web::resource("/account/password").route(web::post().to(change_password)))
	.service(web::resource("/account/security-events").route(web::get().to(security_events)))
	.configure(tokens::configure)
	.configure(totp::configure)
	.configure(trusted::configure);
}
