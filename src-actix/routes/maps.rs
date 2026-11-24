use std::sync::LazyLock;
use std::time::Duration;

use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, get, put, web};
use futures_util::StreamExt;
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};
use serde::Deserialize;
use utoipa::ToSchema;
use obsidian_maps::{DetectedMap, MapKind, MapSettingsUpdate, apply_settings, config_text, detect};
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
	let path = dir.join(&relative);
	let temp = path.with_extension("obsidian-tmp");
	tokio::fs::write(&temp, edited).await.map_err(CommonError::from)?;
	tokio::fs::rename(&temp, &path).await.map_err(CommonError::from)?;
	let map = detect(&dir).await?.into_iter().find(|map| map.kind == kind);
	let map = map.ok_or_else(|| CommonError::NotFound(format!("{} map", kind.label())))?;
	Ok(web::Json(map))
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
	let map = detect(&dir)
		.await?
		.into_iter()
		.find(|map| map.kind == kind && map.settings.enabled)
		.ok_or_else(|| CommonError::NotFound(format!("an enabled {} map", kind.label())))?;
	let url = upstream_url(map.settings.port, map.settings.bind.as_deref(), &tail, req.query_string());
	let mut request = UPSTREAM.get(url);
	for name in FORWARDED_REQUEST_HEADERS {
		if let Some(value) = req.headers().get(name) {
			request = request.header(name, value.as_bytes());
		}
	}
	let upstream = request
		.send()
		.await
		.map_err(|error| CommonError::Conflict(format!("the map is not answering yet: {error}")))?;
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

fn upstream_url(port: u16, bind: Option<&str>, tail: &str, query: &str) -> String {
	let host = match bind {
		Some(address) if !matches!(address, "" | "0.0.0.0" | "::" | "[::]") => address.to_owned(),
		_ => "127.0.0.1".to_owned(),
	};
	let host = if host.contains(':') && !host.starts_with('[') {
		format!("[{host}]")
	} else {
		host
	};
	let query = if query.is_empty() { String::new() } else { format!("?{query}") };
	format!("http://{host}:{port}/{tail}{query}")
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
	fn specific_binds_are_reached_directly() {
		assert_eq!(upstream_url(8080, Some("192.168.1.10"), "", ""), "http://192.168.1.10:8080/");
		assert_eq!(upstream_url(8080, Some("::1"), "", ""), "http://[::1]:8080/");
	}

	#[test]
	fn only_root_relative_locations_are_rewritten() {
		assert_eq!(rewrite_location(b"/index.html", "/api/v1/instances/4/map/bluemap"), b"/api/v1/instances/4/map/bluemap/index.html");
		assert_eq!(rewrite_location(b"https://example.com/", "/prefix"), b"https://example.com/");
	}
}
