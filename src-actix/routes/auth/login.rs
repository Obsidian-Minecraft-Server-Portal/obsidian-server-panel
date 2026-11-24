use actix_web::{HttpRequest, HttpResponse, web};
use obsidian_auth::{LoginOutcome, LoginRequest, UserProfile};
use serde::Serialize;
use utoipa::ToSchema;

use super::support::{SESSION_COOKIE, cleared_session_cookie, client, session_cookie};
use crate::auth::AuthUser;
use crate::email;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
	/// `authenticated`, `totp_required` or `email_code_required`.
	pub status: &'static str,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<UserProfile>,
}

#[utoipa::path(
	post,
	path = "/api/v1/auth/login",
	tag = "auth",
	request_body = LoginRequest,
	responses(
		(status = 200, description = "Login step result; the session cookie is set when authenticated", body = LoginResponse),
		(status = 401, description = "Invalid credentials or verification code"),
		(status = 429, description = "Too many attempts")
	)
)]
pub async fn login(
	state: web::Data<AppState>,
	req: HttpRequest,
	body: web::Json<LoginRequest>,
) -> ApiResult<HttpResponse> {
	let smtp_enabled = email::effective_smtp(&state).await?.is_some();
	let response = match state.auth.login(&body, &client(&req), smtp_enabled).await? {
		LoginOutcome::Authenticated { session_token, user } => {
			return Ok(HttpResponse::Ok()
				.cookie(session_cookie(&state.config, &req, session_token))
				.json(LoginResponse {
					status: "authenticated",
					user: Some(user),
				}));
		}
		LoginOutcome::TotpRequired => LoginResponse {
			status: "totp_required",
			user: None,
		},
		LoginOutcome::EmailCodeRequired { to, code } => {
			email::send_login_code(&state, &to, &code).await?;
			LoginResponse {
				status: "email_code_required",
				user: None,
			}
		}
	};
	Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
	post,
	path = "/api/v1/auth/logout",
	tag = "auth",
	responses((status = 204, description = "Session ended"))
)]
pub async fn logout(state: web::Data<AppState>, req: HttpRequest) -> ApiResult<HttpResponse> {
	if let Some(cookie) = req.cookie(SESSION_COOKIE) {
		let signed_in = state.auth.user_for_session(cookie.value()).await?;
		state.auth.logout(cookie.value()).await?;
		if let Some(user) = signed_in {
			crate::ws::access_changed(&state.bus, user.id);
		}
	}
	Ok(HttpResponse::NoContent()
		.cookie(cleared_session_cookie(&state.config, &req))
		.finish())
}

#[utoipa::path(
	get,
	path = "/api/v1/auth/me",
	tag = "auth",
	responses(
		(status = 200, description = "The signed-in user", body = UserProfile),
		(status = 401, description = "Not signed in")
	)
)]
pub async fn me(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.profile(user.id).await?))
}
