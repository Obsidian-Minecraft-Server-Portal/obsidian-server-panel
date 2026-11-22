use actix_web::{HttpResponse, web};
use std::ops::RangeInclusive;

use chrono_tz::Tz;
use obsidian_common::CommonError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::AdminUser;
use crate::config::{SmtpConfig, SmtpTls};
use crate::email::{self, SMTP_SETTING};
use crate::error::ApiResult;
use crate::routes::auth::support::{PUBLIC_URL_KEY, normalize_public_url};
use crate::state::AppState;

pub const TIMEZONE_KEY: &str = "timezone";
pub const UPNP_DEFAULT_KEY: &str = "upnp_default";
pub const STOP_TIMEOUT_KEY: &str = "stop_timeout_secs";
const DEFAULT_STOP_TIMEOUT_SECS: u64 = 30;
const STOP_TIMEOUT_RANGE: RangeInclusive<u64> = 5..=600;
const ENV_SOURCE: &str = "env";
const PANEL_SOURCE: &str = "panel";

#[derive(Debug, Serialize, ToSchema)]
pub struct SmtpView {
	pub host: String,
	pub port: u16,
	pub username: Option<String>,
	pub from: String,
	pub tls: String,
	pub password_set: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SettingsView {
	pub timezone: String,
	pub upnp_default: bool,
	/// `env` when `OBSIDIAN_UPNP_DEFAULT` or the config file pins it read-only, `panel` when stored in the panel.
	pub upnp_default_source: Option<String>,
	/// Seconds a stopping server gets to save before it is killed.
	pub stop_timeout_secs: u64,
	/// `env` when `OBSIDIAN_STOP_TIMEOUT_SECS` or the config file pins it read-only, `panel` when stored in the panel.
	pub stop_timeout_source: Option<String>,
	pub public_url: Option<String>,
	/// `config` when the public URL comes from the environment or config file and is read-only here.
	pub public_url_source: Option<String>,
	pub smtp: Option<SmtpView>,
	/// `config` when SMTP comes from the environment or config file and is read-only here, `panel` otherwise.
	pub smtp_source: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SettingsUpdate {
	pub timezone: Option<String>,
	pub upnp_default: Option<bool>,
	/// Between 5 and 600 seconds. Ignored when the environment sets it.
	pub stop_timeout_secs: Option<u64>,
	/// Empty string clears the panel-managed value.
	pub public_url: Option<String>,
	/// Omitting `password` keeps the stored one.
	#[schema(value_type = Option<Object>)]
	pub smtp: Option<SmtpConfig>,
}

pub fn validate_timezone(value: &str) -> ApiResult<Tz> {
	value
		.parse::<Tz>()
		.map_err(|_| CommonError::Invalid(format!("unknown timezone {value}")).into())
}

pub fn validate_stop_timeout(secs: u64) -> ApiResult<u64> {
	if STOP_TIMEOUT_RANGE.contains(&secs) {
		Ok(secs)
	} else {
		Err(CommonError::Invalid(format!(
			"stop timeout must be between {} and {} seconds",
			STOP_TIMEOUT_RANGE.start(),
			STOP_TIMEOUT_RANGE.end()
		))
		.into())
	}
}

fn source(from_env: bool, stored: bool) -> Option<String> {
	let label = match (from_env, stored) {
		(true, _) => ENV_SOURCE,
		(false, true) => PANEL_SOURCE,
		(false, false) => return None,
	};
	Some(label.to_owned())
}

/// Copies environment-supplied defaults into the settings table, which the instance manager reads directly.
/// Runs on every boot so an env value is authoritative and cannot silently go stale.
pub async fn seed_from_config(state: &AppState) -> ApiResult<()> {
	if let Some(upnp) = state.config.upnp_default {
		state.auth.set_setting(UPNP_DEFAULT_KEY, bool_value(upnp)).await?;
	}
	if let Some(secs) = state.config.stop_timeout_secs {
		state
			.auth
			.set_setting(STOP_TIMEOUT_KEY, &validate_stop_timeout(secs)?.to_string())
			.await?;
	}
	Ok(())
}

/// Persists panel-managed runtime defaults. Values pinned by the environment are skipped and stay read-only.
/// `stop_timeout_secs` must already be validated with [`validate_stop_timeout`].
pub async fn store_runtime_defaults(
	state: &AppState,
	upnp_default: Option<bool>,
	stop_timeout_secs: Option<u64>,
) -> ApiResult<()> {
	if let Some(upnp) = upnp_default.filter(|_| state.config.upnp_default.is_none()) {
		state.auth.set_setting(UPNP_DEFAULT_KEY, bool_value(upnp)).await?;
	}
	if let Some(secs) = stop_timeout_secs.filter(|_| state.config.stop_timeout_secs.is_none()) {
		state.auth.set_setting(STOP_TIMEOUT_KEY, &secs.to_string()).await?;
	}
	Ok(())
}

fn bool_value(value: bool) -> &'static str {
	if value { "true" } else { "false" }
}

async fn view(state: &AppState) -> ApiResult<SettingsView> {
	let stored: Option<SmtpConfig> = state.auth.secret_setting(SMTP_SETTING).await?;
	let (smtp, smtp_source) = match (&state.config.smtp, stored) {
		(Some(config), _) => (Some(config.clone()), Some("config")),
		(None, Some(stored)) => (Some(stored), Some("panel")),
		(None, None) => (None, None),
	};
	let timezone = state
		.auth
		.setting(TIMEZONE_KEY)
		.await?
		.unwrap_or_else(|| state.config.timezone.clone());
	let stored_upnp = state.auth.setting(UPNP_DEFAULT_KEY).await?;
	let upnp_default = state
		.config
		.upnp_default
		.unwrap_or_else(|| stored_upnp.as_deref() == Some("true"));
	let upnp_default_source = source(state.config.upnp_default.is_some(), stored_upnp.is_some());
	let stored_stop = state.auth.setting(STOP_TIMEOUT_KEY).await?;
	let stop_timeout_source = source(state.config.stop_timeout_secs.is_some(), stored_stop.is_some());
	let stop_timeout_secs = state.config.stop_timeout_secs.unwrap_or_else(|| {
		stored_stop
			.as_deref()
			.and_then(|value| value.trim().parse().ok())
			.unwrap_or(DEFAULT_STOP_TIMEOUT_SECS)
	});
	let (public_url, public_url_source) = match &state.config.public_url {
		Some(url) => (Some(url.trim_end_matches('/').to_owned()), Some("config")),
		None => {
			let stored = state.auth.setting(PUBLIC_URL_KEY).await?;
			let source = stored.as_ref().map(|_| "panel");
			(stored, source)
		}
	};
	Ok(SettingsView {
		timezone,
		upnp_default,
		upnp_default_source,
		stop_timeout_secs,
		stop_timeout_source,
		public_url,
		public_url_source: public_url_source.map(str::to_owned),
		smtp: smtp.map(smtp_view),
		smtp_source: smtp_source.map(str::to_owned),
	})
}

fn smtp_view(smtp: SmtpConfig) -> SmtpView {
	let tls = match smtp.tls {
		SmtpTls::None => "none",
		SmtpTls::Starttls => "starttls",
		SmtpTls::Tls => "tls",
	};
	SmtpView {
		host: smtp.host,
		port: smtp.port,
		username: smtp.username,
		from: smtp.from,
		tls: tls.to_owned(),
		password_set: smtp.password.is_some(),
	}
}

#[utoipa::path(
	get,
	path = "/api/v1/settings",
	tag = "settings",
	responses((status = 200, description = "Panel settings", body = SettingsView), (status = 403, description = "Administrator only"))
)]
pub async fn get_settings(state: web::Data<AppState>, _admin: AdminUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(view(&state).await?))
}

#[utoipa::path(
	put,
	path = "/api/v1/settings",
	tag = "settings",
	request_body = SettingsUpdate,
	responses((status = 200, description = "Updated settings", body = SettingsView), (status = 400, description = "Invalid value"))
)]
pub async fn update_settings(
	state: web::Data<AppState>,
	_admin: AdminUser,
	body: web::Json<SettingsUpdate>,
) -> ApiResult<HttpResponse> {
	let body = body.into_inner();
	if let Some(timezone) = &body.timezone {
		let tz = validate_timezone(timezone)?;
		state.auth.set_setting(TIMEZONE_KEY, timezone).await?;
		state.scheduler.set_tz(tz).await?;
	}
	let stop_timeout_secs = body.stop_timeout_secs.map(validate_stop_timeout).transpose()?;
	store_runtime_defaults(&state, body.upnp_default, stop_timeout_secs).await?;
	match body.public_url.as_deref().map(str::trim) {
		Some("") => state.auth.delete_setting(PUBLIC_URL_KEY).await?,
		Some(url) => {
			state
				.auth
				.set_setting(PUBLIC_URL_KEY, &normalize_public_url(url)?)
				.await?
		}
		None => {}
	}
	if let Some(mut smtp) = body.smtp {
		if smtp.password.is_none() {
			let stored: Option<SmtpConfig> = state.auth.secret_setting(SMTP_SETTING).await?;
			smtp.password = stored.and_then(|stored| stored.password);
		}
		state.auth.set_secret_setting(SMTP_SETTING, &smtp).await?;
	}
	Ok(HttpResponse::Ok().json(view(&state).await?))
}

#[utoipa::path(
	delete,
	path = "/api/v1/settings/smtp",
	tag = "settings",
	responses((status = 204, description = "Panel-managed SMTP removed"))
)]
pub async fn remove_smtp(state: web::Data<AppState>, _admin: AdminUser) -> ApiResult<HttpResponse> {
	state.auth.delete_setting(SMTP_SETTING).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/settings/smtp/test",
	tag = "settings",
	responses(
		(status = 204, description = "Test email sent to the administrator's address"),
		(status = 400, description = "SMTP is not configured or the account has no email address")
	)
)]
pub async fn test_smtp(state: web::Data<AppState>, admin: AdminUser) -> ApiResult<HttpResponse> {
	let profile = state.auth.profile(admin.id).await?;
	let to = profile
		.email
		.ok_or_else(|| CommonError::Invalid("add an email address to your account first".into()))?;
	email::send_test(&state, &to).await?;
	Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/settings")
			.route(web::get().to(get_settings))
			.route(web::put().to(update_settings)),
	)
	.service(web::resource("/settings/smtp").route(web::delete().to(remove_smtp)))
	.service(web::resource("/settings/smtp/test").route(web::post().to(test_smtp)));
}
