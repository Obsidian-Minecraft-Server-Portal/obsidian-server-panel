use actix_web::{HttpRequest, HttpResponse, web};
use obsidian_common::CommonError;
use serde::Deserialize;
use utoipa::ToSchema;

use super::support::{client, public_url};
use crate::email;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
	pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
	pub token: String,
	pub password: String,
}

#[utoipa::path(
	post,
	path = "/api/v1/auth/forgot-password",
	tag = "auth",
	request_body = ForgotPasswordRequest,
	responses(
		(status = 202, description = "Accepted; a link is sent if the address belongs to an account"),
		(status = 400, description = "Password reset needs SMTP and the public URL to be configured"),
		(status = 429, description = "Too many attempts")
	)
)]
pub async fn forgot_password(
	state: web::Data<AppState>,
	body: web::Json<ForgotPasswordRequest>,
) -> ApiResult<HttpResponse> {
	if email::effective_smtp(&state).await?.is_none() {
		return Err(CommonError::Invalid("password reset requires SMTP to be configured".into()).into());
	}
	let Some(base) = public_url(&state).await? else {
		return Err(CommonError::Invalid("password reset requires the panel's public URL to be configured".into()).into());
	};
	if let Some(request) = state.auth.request_password_reset(&body.email).await? {
		let link = format!("{base}/reset-password?token={}", request.token);
		let state = state.clone();
		actix_web::rt::spawn(async move {
			if let Err(error) = email::send_password_reset(&state, &request.to, &link).await {
				tracing::error!(%error, "failed to send password reset email");
			}
		});
	}
	Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/auth/reset-password",
	tag = "auth",
	request_body = ResetPasswordRequest,
	responses(
		(status = 204, description = "Password changed; every session was signed out"),
		(status = 400, description = "Link is invalid, expired or already used")
	)
)]
pub async fn reset_password(
	state: web::Data<AppState>,
	req: HttpRequest,
	body: web::Json<ResetPasswordRequest>,
) -> ApiResult<HttpResponse> {
	state
		.auth
		.complete_password_reset(&body.token, &body.password, &client(&req))
		.await?;
	Ok(HttpResponse::NoContent().finish())
}
