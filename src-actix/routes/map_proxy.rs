use std::net::{IpAddr, Ipv4Addr};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use futures_util::StreamExt;
use obsidian_auth::{Permission, SecretBox};
use obsidian_common::{CommonError, InstanceId, UserId};
use obsidian_maps::{MapKind, config_text, read_settings};
use obsidian_runner::RunState;
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

const TOKEN_CONTEXT: &str = "obsidian 2026-11 map proxy token";
const TOKEN_TTL_SECS: i64 = 60 * 60;
const LOCAL_ADDRESS_TTL: Duration = Duration::from_secs(30);
const SANDBOX_POLICY: &str = "sandbox allow-scripts allow-popups allow-forms allow-pointer-lock";
const STORAGE_SHIM: &str = "<script>(()=>{const store=()=>{const d=new Map();return{get length(){return d.size},key:i=>[...d.keys()][i]??null,getItem:k=>d.has(String(k))?d.get(String(k)):null,setItem:(k,v)=>{d.set(String(k),String(v))},removeItem:k=>{d.delete(String(k))},clear:()=>d.clear()}};for(const name of[\"localStorage\",\"sessionStorage\"]){try{window[name].length}catch{Object.defineProperty(window,name,{value:store(),configurable:true})}}})()</script>";
const DROPPED_RESPONSE_HEADERS: [&str; 15] = [
	"connection",
	"keep-alive",
	"proxy-authenticate",
	"proxy-authorization",
	"te",
	"trailer",
	"transfer-encoding",
	"upgrade",
	"set-cookie",
	"content-security-policy",
	"content-security-policy-report-only",
	"x-frame-options",
	"access-control-allow-origin",
	"access-control-allow-credentials",
	"referrer-policy",
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

static LOCAL_ADDRESSES: Mutex<Option<(Instant, Vec<IpAddr>)>> = Mutex::new(None);

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(create_map_session).service(map_root).service(proxy_map);
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MapSession {
	/// Root of the proxied map, ending in `/`. Requests under it need no cookie, so a sandboxed frame can load it.
	pub path: String,
	/// Unix seconds after which the path stops working; ask for a new session before then.
	pub expires_at: i64,
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/maps/{kind}/session",
	tag = "instances",
	params(
		("id" = i64, Path, description = "Instance id"),
		("kind" = String, Path, description = "bluemap, dynmap, squaremap or pl3xmap"),
	),
	responses(
		(status = 200, description = "A short-lived path to the map that a sandboxed frame can load", body = MapSession),
		(status = 403, description = "Missing console.read permission"),
	)
)]
#[post("/instances/{id}/maps/{kind}/session")]
pub async fn create_map_session(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, MapKind)>,
) -> ApiResult<web::Json<MapSession>> {
	let (id, kind) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::CONSOLE_READ).await?;
	let expires_at = chrono::Utc::now().timestamp() + TOKEN_TTL_SECS;
	let token = sign_token(state.auth.secrets(), id, kind, user.id, expires_at);
	Ok(web::Json(MapSession { path: format!("{}/", token_prefix(id, kind, &token)), expires_at }))
}

#[get("/instances/{id}/map/{kind}/t/{token}")]
pub async fn map_root(path: web::Path<(InstanceId, MapKind, String)>) -> HttpResponse {
	let (id, kind, token) = path.into_inner();
	HttpResponse::PermanentRedirect()
		.insert_header((header::LOCATION, format!("{}/", token_prefix(id, kind, &token))))
		.finish()
}

#[get("/instances/{id}/map/{kind}/t/{token}/{tail:.*}")]
pub async fn proxy_map(
	state: web::Data<AppState>,
	req: HttpRequest,
	path: web::Path<(InstanceId, MapKind, String, String)>,
) -> ApiResult<HttpResponse> {
	let (id, kind, token, tail) = path.into_inner();
	let now = chrono::Utc::now().timestamp();
	let user_id = verify_token(state.auth.secrets(), id, kind, &token, now)
		.ok_or_else(|| CommonError::Unauthorized("the map link expired; reopen the Map tab".into()))?;
	let user = state
		.auth
		.user(user_id)
		.await?
		.ok_or_else(|| CommonError::Unauthorized("the map link belongs to a removed user".into()))?;
	require_instance_perm(&state, &AuthUser(user), id, Permission::CONSOLE_READ).await?;
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
	let prefix = token_prefix(id, kind, &token);
	let status = actix_web::http::StatusCode::from_u16(upstream.status().as_u16())
		.map_err(|error| CommonError::Other(format!("the map sent an invalid status: {error}")))?;
	let page = status == actix_web::http::StatusCode::OK && is_html(&upstream);
	let mut response = HttpResponse::build(status);
	for (name, value) in upstream.headers() {
		let name_text = name.as_str();
		if DROPPED_RESPONSE_HEADERS.contains(&name_text) || (page && name_text == "content-length") {
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
	isolate(&mut response);
	if page {
		let page = upstream
			.bytes()
			.await
			.map_err(|error| CommonError::Other(format!("the map page could not be read: {error}")))?;
		return Ok(response.body(with_storage_shim(&page)));
	}
	Ok(response.streaming(upstream.bytes_stream().map(|chunk| chunk.map_err(std::io::Error::other))))
}

fn is_html(upstream: &reqwest::Response) -> bool {
	upstream
		.headers()
		.get(reqwest::header::CONTENT_TYPE)
		.and_then(|value| value.to_str().ok())
		.is_some_and(|value| value.trim_start().to_ascii_lowercase().starts_with("text/html"))
}

// An opaque origin throws on Web Storage, and map apps read it unguarded at startup, so pages get a per-load in-memory store.
fn with_storage_shim(page: &[u8]) -> Vec<u8> {
	let insert_at = find_ascii_case_insensitive(page, b"<head")
		.and_then(|start| page[start..].iter().position(|&byte| byte == b'>').map(|end| start + end + 1))
		.unwrap_or(0);
	[&page[..insert_at], STORAGE_SHIM.as_bytes(), &page[insert_at..]].concat()
}

fn find_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> Option<usize> {
	haystack.windows(needle.len()).position(|window| window.eq_ignore_ascii_case(needle))
}

// The map runs as an opaque origin: no panel cookies, no parent DOM, and its own fetches are cross-origin, so they need CORS.
fn isolate(response: &mut actix_web::HttpResponseBuilder) {
	response
		.insert_header((header::CONTENT_SECURITY_POLICY, SANDBOX_POLICY))
		.insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
		.insert_header((header::REFERRER_POLICY, "no-referrer"));
}

fn unavailable_page(label: &str, reason: &str) -> HttpResponse {
	let reason = reason.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
	let body = format!(
		"<!doctype html><meta charset=\"utf-8\"><title>{label} map</title><body style=\"margin:0;display:grid;place-items:center;height:100vh;background:#0a0609;color:#c9c2cc;font:14px system-ui,sans-serif\"><div style=\"max-width:32rem;padding:24px;text-align:center\"><p style=\"font-size:16px;color:#f2eef4\">The {label} map is not answering yet.</p><p>It is still starting, or its web server could not start. Check the server console for {label} messages.</p><p style=\"font:12px monospace;color:#7d7480\">{reason}</p></div></body>"
	);
	let mut response = HttpResponse::ServiceUnavailable();
	isolate(&mut response);
	response.content_type("text/html; charset=utf-8").body(body)
}

fn token_prefix(id: InstanceId, kind: MapKind, token: &str) -> String {
	format!("/api/v1/instances/{id}/map/{}/t/{token}", kind.key())
}

fn token_message(id: InstanceId, kind: MapKind, user: UserId, expires_at: i64) -> String {
	format!("{id}:{}:{user}:{expires_at}", kind.key())
}

fn sign_token(secrets: &SecretBox, id: InstanceId, kind: MapKind, user: UserId, expires_at: i64) -> String {
	let tag = secrets.mac(TOKEN_CONTEXT, token_message(id, kind, user, expires_at).as_bytes());
	format!("{user}.{expires_at}.{tag}")
}

fn verify_token(secrets: &SecretBox, id: InstanceId, kind: MapKind, token: &str, now: i64) -> Option<UserId> {
	let mut parts = token.splitn(3, '.');
	let user: UserId = parts.next()?.parse().ok()?;
	let expires_at: i64 = parts.next()?.parse().ok()?;
	let tag = parts.next()?;
	let valid = expires_at > now && secrets.verify_mac(TOKEN_CONTEXT, token_message(id, kind, user, expires_at).as_bytes(), tag);
	valid.then_some(user)
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
		Some(address) if !address.is_unspecified() && (address.is_loopback() || is_local_address(address)) => address,
		_ => IpAddr::V4(Ipv4Addr::LOCALHOST),
	};
	match address {
		IpAddr::V4(address) => address.to_string(),
		IpAddr::V6(address) => format!("[{address}]"),
	}
}

fn is_local_address(address: IpAddr) -> bool {
	let mut cache = LOCAL_ADDRESSES.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
	if cache.as_ref().is_none_or(|(read_at, _)| read_at.elapsed() > LOCAL_ADDRESS_TTL) {
		let addresses = local_ip_address::list_afinet_netifas()
			.map(|interfaces| interfaces.into_iter().map(|(_, address)| address).collect())
			.unwrap_or_default();
		*cache = Some((Instant::now(), addresses));
	}
	cache.as_ref().is_some_and(|(_, addresses)| addresses.contains(&address))
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
	use obsidian_auth::SecretBox;
	use obsidian_maps::MapKind;

	use super::{STORAGE_SHIM, rewrite_location, sign_token, upstream_url, verify_token, with_storage_shim};

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
		let prefix = "/api/v1/instances/4/map/bluemap/t/1.2.ab";
		assert_eq!(rewrite_location(b"/index.html", prefix), b"/api/v1/instances/4/map/bluemap/t/1.2.ab/index.html");
		assert_eq!(rewrite_location(b"https://example.com/", prefix), b"https://example.com/");
	}

	#[test]
	fn the_storage_shim_runs_before_page_scripts() {
		let shimmed = with_storage_shim(b"<!doctype html><HTML><Head lang=\"en\"><script src=app.js></script>");
		let text = String::from_utf8(shimmed).unwrap();
		assert!(text.starts_with(&format!("<!doctype html><HTML><Head lang=\"en\">{STORAGE_SHIM}<script src=app.js>")));
		assert!(String::from_utf8(with_storage_shim(b"<p>no head</p>")).unwrap().starts_with(STORAGE_SHIM));
	}

	#[test]
	fn tokens_verify_for_their_map_until_they_expire() {
		let secrets = SecretBox::from_key([3; 32]);
		let token = sign_token(&secrets, 4, MapKind::Bluemap, 9, 1_000);
		assert_eq!(verify_token(&secrets, 4, MapKind::Bluemap, &token, 999), Some(9));
		assert_eq!(verify_token(&secrets, 4, MapKind::Bluemap, &token, 1_000), None);
		assert_eq!(verify_token(&secrets, 5, MapKind::Bluemap, &token, 999), None);
		assert_eq!(verify_token(&secrets, 4, MapKind::Dynmap, &token, 999), None);
		assert_eq!(verify_token(&SecretBox::from_key([4; 32]), 4, MapKind::Bluemap, &token, 999), None);
	}

	#[test]
	fn tampered_tokens_are_rejected() {
		let secrets = SecretBox::from_key([3; 32]);
		let token = sign_token(&secrets, 4, MapKind::Bluemap, 9, 1_000);
		let tag = token.rsplit('.').next().unwrap();
		assert_eq!(verify_token(&secrets, 4, MapKind::Bluemap, &format!("1.1000.{tag}"), 999), None);
		assert_eq!(verify_token(&secrets, 4, MapKind::Bluemap, &format!("9.5000.{tag}"), 999), None);
		assert_eq!(verify_token(&secrets, 4, MapKind::Bluemap, "9.1000", 999), None);
		assert_eq!(verify_token(&secrets, 4, MapKind::Bluemap, "garbage", 999), None);
	}
}
