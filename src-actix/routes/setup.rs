use actix_web::{HttpRequest, HttpResponse, web};
use obsidian_auth::{NewUser, UserProfile};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::config::SmtpConfig;
use crate::email::SMTP_SETTING;
use crate::error::ApiResult;
use crate::routes::auth::support::{PUBLIC_URL_KEY, client, normalize_public_url, request_origin, session_cookie};
use crate::routes::settings::{TIMEZONE_KEY, store_runtime_defaults, validate_stop_timeout, validate_timezone};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetupRequest {
	pub username: String,
	pub email: Option<String>,
	pub password: String,
	pub timezone: Option<String>,
	pub upnp_default: Option<bool>,
	/// Between 5 and 600 seconds.
	pub stop_timeout_secs: Option<u64>,
	/// Defaults to the origin the wizard was opened from.
	pub public_url: Option<String>,
	#[schema(value_type = Option<Object>)]
	pub smtp: Option<SmtpConfig>,
}

#[utoipa::path(
	post,
	path = "/api/v1/setup",
	tag = "setup",
	request_body = SetupRequest,
	responses(
		(status = 201, description = "Administrator created and signed in", body = UserProfile),
		(status = 400, description = "Invalid details"),
		(status = 409, description = "Setup has already been completed")
	)
)]
pub async fn complete_setup(
	state: web::Data<AppState>,
	req: HttpRequest,
	body: web::Json<SetupRequest>,
) -> ApiResult<HttpResponse> {
	let body = body.into_inner();
	let tz = body.timezone.as_deref().map(validate_timezone).transpose()?;
	let stop_timeout_secs = body.stop_timeout_secs.map(validate_stop_timeout).transpose()?;
	let public_url = match (&state.config.public_url, &body.public_url) {
		(Some(_), _) => None,
		(None, Some(url)) => Some(normalize_public_url(url)?),
		(None, None) => normalize_public_url(&request_origin(&req)).ok(),
	};
	let user = state
		.auth
		.create_first_admin(&NewUser {
			username: body.username,
			email: body.email.filter(|email| !email.is_empty()),
			password: body.password,
			is_admin: true,
			instance_limit: None,
		})
		.await?;
	if let (Some(timezone), Some(tz)) = (&body.timezone, tz) {
		state.auth.set_setting(TIMEZONE_KEY, timezone).await?;
		state.scheduler.set_tz(tz).await?;
	}
	store_runtime_defaults(&state, body.upnp_default, stop_timeout_secs).await?;
	if let Some(url) = &public_url {
		state.auth.set_setting(PUBLIC_URL_KEY, url).await?;
	}
	if let Some(smtp) = &body.smtp {
		state.auth.set_secret_setting(SMTP_SETTING, smtp).await?;
	}
	let client = client(&req);
	state.auth.trust_ip(user.id, &client).await?;
	let token = state.auth.start_session(user.id, &client).await?;
	Ok(HttpResponse::Created()
		.cookie(session_cookie(&state.config, &req, token))
		.json(user))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("/setup").route(web::post().to(complete_setup)));
}
