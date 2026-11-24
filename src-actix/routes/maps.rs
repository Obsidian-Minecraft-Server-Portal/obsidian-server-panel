use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, get, put, web};
use futures_util::StreamExt;
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};
use serde::Deserialize;
use utoipa::ToSchema;
use obsidian_maps::{
	ConfigEntry, DetectedMap, MapKind, MapSettingsUpdate, apply_settings, config_text, detect, port_path, read_settings,
	scalar_entries, set_scalar_entry,
};
use obsidian_runner::RunState;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

const HOP_BY_HOP: [&str; 8] = [
	"connection",
	"keep-alive",
	"proxy-authenticate",
	"proxy-authorization",
	"te",
	"trailer",
	"transfer-encoding",
	"upgrade",
];
const FORWARDED_REQUEST_HEADERS: [&str; 7] = [
	"accept",
	"accept-language",
	"if-modified-since",
	"if-none-match",
	"if-range",
	"range",
	"user-agent",
];

static UPSTREAM: LazyLock<reqwest::Client> = LazyLock::new(|| {
	reqwest::Client::builder()
		.redirect(reqwest::redirect::Policy::none())
		.connect_timeout(Duration::from_secs(5))
		.build()
		.expect("the map proxy client builds with static settings")
});

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_maps)
		.service(update_map)
		.service(map_root)
		.service(set_map_upnp)
		.service(list_entries)
		.service(update_entry)
		.service(proxy_map);
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/maps",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Web maps found for this instance", body = Vec<DetectedMap>),
		(status = 403, description = "Missing console.read permission"),
	)
)]
#[get("/instances/{id}/maps")]
pub async fn list_maps(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<web::Json<Vec<DetectedMap>>> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	Ok(web::Json(detect(&state.paths.instance_dir(id)).await?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/maps/{kind}",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("kind" = String, Path, description = "bluemap, dynmap, squaremap or pl3xmap"),
	),
	request_body = MapSettingsUpdate,
	responses(
		(status = 200, description = "Updated map settings; the map reads them on its next start", body = DetectedMap),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "The map cannot be edited here, or its port matches the server port"),
	)
)]
#[put("/instances/{id}/maps/{kind}")]
pub async fn update_map(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, MapKind)>,
	web::Json(update): web::Json<MapSettingsUpdate>,
) -> ApiResult<web::Json<DetectedMap>> {
	let (id, kind) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let instance = state.instances.get(id).await?;
	if update.port.is_some_and(|port| port == instance.port) {
		return Err(CommonError::Conflict("the map port cannot be the same as the server port".into()).into());
	}
	let dir = state.paths.instance_dir(id);
	let Some((relative, text)) = config_text(&dir, kind).await? else {
		return Err(CommonError::Conflict(format!("start the server once so {} can create its config", kind.label())).into());
	};
	let edited = apply_settings(kind, &text, &update)?;
	save_config(&dir, &relative, &edited).await?;
	let map = detect(&dir).await?.into_iter().find(|map| map.kind == kind);
	let map = map.ok_or_else(|| CommonError::NotFound(format!("{} map", kind.label())))?;
	Ok(web::Json(map))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EntryUpdate {
	/// Dotted path from the config root, as listed by the entries endpoint.
	pub path: String,
	pub value: String,
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/maps/{kind}/entries",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("kind" = String, Path, description = "dynmap, squaremap or pl3xmap"),
	),
	responses(
		(status = 200, description = "Every scalar setting in the map config", body = Vec<ConfigEntry>),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "The config is read-only here or does not exist yet"),
	)
)]
#[get("/instances/{id}/maps/{kind}/entries")]
pub async fn list_entries(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, MapKind)>,
) -> ApiResult<web::Json<Vec<ConfigEntry>>> {
	let (id, kind) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let Some((_, text)) = config_text(&state.paths.instance_dir(id), kind).await? else {
		return Err(CommonError::Conflict(format!("start the server once so {} can create its config", kind.label())).into());
	};
	Ok(web::Json(scalar_entries(kind, &text)?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/maps/{kind}/entries",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("kind" = String, Path, description = "dynmap, squaremap or pl3xmap"),
	),
	request_body = EntryUpdate,
	responses(
		(status = 200, description = "Updated settings; the map reads them on its next start", body = Vec<ConfigEntry>),
		(status = 403, description = "Missing settings.edit permission"),
		(status = 409, description = "The value clashes with the server port or the config is read-only"),
	)
)]
#[put("/instances/{id}/maps/{kind}/entries")]
pub async fn update_entry(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, MapKind)>,
	web::Json(update): web::Json<EntryUpdate>,
) -> ApiResult<web::Json<Vec<ConfigEntry>>> {
	let (id, kind) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	if update.path == port_path(kind) {
		let port: u16 = update
			.value
			.trim()
			.parse()
			.map_err(|_| CommonError::Invalid(format!("{} is not a valid port", update.value)))?;
		if port == state.instances.get(id).await?.port {
			return Err(CommonError::Conflict("the map port cannot be the same as the server port".into()).into());
		}
	}
	let dir = state.paths.instance_dir(id);
	let Some((relative, text)) = config_text(&dir, kind).await? else {
		return Err(CommonError::Conflict(format!("start the server once so {} can create its config", kind.label())).into());
	};
	let edited = set_scalar_entry(kind, &text, &update.path, &update.value)?;
	save_config(&dir, &relative, &edited).await?;
	Ok(web::Json(scalar_entries(kind, &edited)?))
}

async fn save_config(dir: &Path, relative: &str, text: &str) -> ApiResult<()> {
	let path = dir.join(relative);
	let temp = path.with_extension("obsidian-tmp");
	tokio::fs::write(&temp, text).await.map_err(CommonError::from)?;
	tokio::fs::rename(&temp, &path).await.map_err(CommonError::from)?;
	Ok(())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MapUpnpUpdate {
	pub enabled: bool,
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/map-upnp",
	tag = "instances",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = MapUpnpUpdate,
	responses(
		(status = 204, description = "Forwarding of the web map ports is updated"),
		(status = 403, description = "Missing settings.edit permission"),
	)
)]
#[put("/instances/{id}/map-upnp")]
pub async fn set_map_upnp(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	web::Json(update): web::Json<MapUpnpUpdate>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	state.instances.set_map_upnp(id, update.enabled).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[get("/instances/{id}/map/{kind}")]
pub async fn map_root(path: web::Path<(InstanceId, MapKind)>) -> HttpResponse {
	let (id, kind) = path.into_inner();
	HttpResponse::PermanentRedirect()
		.insert_header((header::LOCATION, format!("/api/v1/instances/{id}/map/{}/", kind.key())))
		.finish()
}

#[get("/instances/{id}/map/{kind}/{tail:.*}")]
pub async fn proxy_map(
	state: web::Data<AppState>,
	user: AuthUser,
	req: HttpRequest,
	path: web::Path<(InstanceId, MapKind, String)>,
) -> ApiResult<HttpResponse> {
	let (id, kind, tail) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	if !matches!(state.instances.status(id).await?.state, RunState::Running) {
		return Err(CommonError::Conflict("start the server to open its map".into()).into());
	}
	let dir = state.paths.instance_dir(id);
	let settings = match config_text(&dir, kind).await? {
		Some((_, text)) => read_settings(kind, &text)?,
		None => return Err(CommonError::NotFound(format!("{} map", kind.label())).into()),
	};
	if !settings.enabled {
		return Err(CommonError::NotFound(format!("an enabled {} map", kind.label())).into());
	}
	let url = upstream_url(settings.port, settings.bind.as_deref(), &tail, req.query_string());
	let mut request = UPSTREAM.get(url);
	for name in FORWARDED_REQUEST_HEADERS {
		if let Some(value) = req.headers().get(name) {
			request = request.header(name, value.as_bytes());
		}
	}
	let upstream = match request.send().await {
		Ok(upstream) => upstream,
		Err(error) => return Ok(unavailable_page(kind.label(), &error.to_string())),
	};
	let prefix = format!("/api/v1/instances/{id}/map/{}", kind.key());
	let status = actix_web::http::StatusCode::from_u16(upstream.status().as_u16())
		.map_err(|error| CommonError::Other(format!("the map sent an invalid status: {error}")))?;
	let mut response = HttpResponse::build(status);
	for (name, value) in upstream.headers() {
		let name_text = name.as_str();
		if HOP_BY_HOP.contains(&name_text) || name_text == "set-cookie" {
			continue;
		}
		let value = if name_text == "location" {
			rewrite_location(value.as_bytes(), &prefix)
		} else {
			value.as_bytes().to_vec()
		};
		if let (Ok(name), Ok(value)) = (header::HeaderName::from_bytes(name_text.as_bytes()), header::HeaderValue::from_bytes(&value)) {
			response.append_header((name, value));
		}
	}
	Ok(response.streaming(upstream.bytes_stream().map(|chunk| chunk.map_err(std::io::Error::other))))
}

fn unavailable_page(label: &str, reason: &str) -> HttpResponse {
	let reason = reason.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
	let body = format!(
		"<!doctype html><meta charset=\"utf-8\"><title>{label} map</title><body style=\"margin:0;display:grid;place-items:center;height:100vh;background:#0a0609;color:#c9c2cc;font:14px system-ui,sans-serif\"><div style=\"max-width:32rem;padding:24px;text-align:center\"><p style=\"font-size:16px;color:#f2eef4\">The {label} map is not answering yet.</p><p>It is still starting, or its web server could not start. Check the server console for {label} messages.</p><p style=\"font:12px monospace;color:#7d7480\">{reason}</p></div></body>"
	);
	HttpResponse::ServiceUnavailable().content_type("text/html; charset=utf-8").body(body)
}


fn upstream_url(port: u16, bind: Option<&str>, tail: &str, query: &str) -> String {
	let host = upstream_host(bind);
	let query = if query.is_empty() { String::new() } else { format!("?{query}") };
	format!("http://{host}:{port}/{tail}{query}")
}

// Only loopback or an address this machine owns is ever contacted, so a bind setting cannot point the proxy at other hosts.
fn upstream_host(bind: Option<&str>) -> String {
	let bound = bind.and_then(|bind| bind.parse::<IpAddr>().ok());
	let address = match bound {
		Some(address) if !address.is_unspecified() && (address.is_loopback() || local_addresses().contains(&address)) => address,
		_ => IpAddr::V4(Ipv4Addr::LOCALHOST),
	};
	match address {
		IpAddr::V4(address) => address.to_string(),
		IpAddr::V6(address) => format!("[{address}]"),
	}
}

fn local_addresses() -> Vec<IpAddr> {
	local_ip_address::list_afinet_netifas()
		.map(|interfaces| interfaces.into_iter().map(|(_, address)| address).collect())
		.unwrap_or_default()
}

fn rewrite_location(location: &[u8], prefix: &str) -> Vec<u8> {
	if location.starts_with(b"/") {
		[prefix.as_bytes(), location].concat()
	} else {
		location.to_vec()
	}
}

#[cfg(test)]
mod tests {
	use super::{rewrite_location, upstream_url};

	#[test]
	fn wildcard_binds_are_reached_through_loopback() {
		assert_eq!(upstream_url(8100, None, "", ""), "http://127.0.0.1:8100/");
		assert_eq!(upstream_url(8100, Some("0.0.0.0"), "maps/x.json", "v=2"), "http://127.0.0.1:8100/maps/x.json?v=2");
		assert_eq!(upstream_url(8100, Some("::"), "", ""), "http://127.0.0.1:8100/");
	}

	#[test]
	fn loopback_binds_are_reached_directly() {
		assert_eq!(upstream_url(8080, Some("127.0.0.2"), "", ""), "http://127.0.0.2:8080/");
		assert_eq!(upstream_url(8080, Some("::1"), "", ""), "http://[::1]:8080/");
		assert_eq!(upstream_url(8080, Some("203.0.113.7"), "", ""), "http://127.0.0.1:8080/");
	}

	#[test]
	fn only_root_relative_locations_are_rewritten() {
		assert_eq!(rewrite_location(b"/index.html", "/api/v1/instances/4/map/bluemap"), b"/api/v1/instances/4/map/bluemap/index.html");
		assert_eq!(rewrite_location(b"https://example.com/", "/prefix"), b"https://example.com/");
	}
}
