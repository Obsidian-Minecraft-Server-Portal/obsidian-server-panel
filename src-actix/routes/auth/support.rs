use std::net::{IpAddr, SocketAddr};

use actix_web::cookie::time::Duration as CookieDuration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::Uri;
use actix_web::http::header::{HOST, USER_AGENT};
use actix_web::{HttpRequest, web};
use ipnet::IpNet;
use obsidian_auth::{ClientInfo, SESSION_TTL_SECS};
use obsidian_common::CommonError;

use crate::config::Config;
use crate::error::ApiResult;
use crate::state::AppState;

pub const SESSION_COOKIE: &str = "obsidian_session";
pub const PUBLIC_URL_KEY: &str = "public_url";
const USER_AGENT_MAX_CHARS: usize = 256;
const FORWARDED: &str = "forwarded";
const X_FORWARDED_FOR: &str = "x-forwarded-for";
const X_FORWARDED_PROTO: &str = "x-forwarded-proto";

pub fn client(req: &HttpRequest) -> ClientInfo {
	let ip = client_ip(req).map_or_else(|| "unknown".to_owned(), |ip| ip.to_string());
	let user_agent = req
		.headers()
		.get(USER_AGENT)
		.and_then(|value| value.to_str().ok())
		.map(|value| value.chars().take(USER_AGENT_MAX_CHARS).collect());
	ClientInfo { ip, user_agent }
}

/// The address a request is attributed to. Forwarding headers are read only when the TCP peer is a trusted proxy.
pub fn client_ip(req: &HttpRequest) -> Option<IpAddr> {
	let peer = req.peer_addr()?.ip().to_canonical();
	let trusted = trusted_proxies(req);
	if !is_trusted(trusted, peer) {
		return Some(peer);
	}
	Some(resolve_client(&forwarded_chain(req), trusted, peer))
}

fn trusted_proxies(req: &HttpRequest) -> &[IpNet] {
	req.app_data::<web::Data<AppState>>()
		.map(|state| state.config.trusted_proxies.as_slice())
		.unwrap_or_default()
}

fn from_trusted_proxy(req: &HttpRequest, trusted: &[IpNet]) -> bool {
	req.peer_addr()
		.is_some_and(|addr| is_trusted(trusted, addr.ip().to_canonical()))
}

/// Walks the proxy chain from the right and returns the first hop that is not a trusted proxy.
fn resolve_client(chain: &[IpAddr], trusted: &[IpNet], peer: IpAddr) -> IpAddr {
	chain
		.iter()
		.rev()
		.copied()
		.find(|ip| !is_trusted(trusted, *ip))
		.or_else(|| chain.first().copied())
		.unwrap_or(peer)
}

fn is_trusted(trusted: &[IpNet], ip: IpAddr) -> bool {
	trusted.iter().any(|net| net.contains(&ip))
}

/// `X-Forwarded-For` wins because most proxies append only to it and pass a client-sent `Forwarded` through untouched.
fn forwarded_chain(req: &HttpRequest) -> Vec<IpAddr> {
	if req.headers().contains_key(X_FORWARDED_FOR) {
		return header_list(req, X_FORWARDED_FOR).filter_map(parse_hop).collect();
	}
	header_list(req, FORWARDED).filter_map(forwarded_for).collect()
}

fn header_list<'a>(req: &'a HttpRequest, name: &'static str) -> impl Iterator<Item = &'a str> {
	req.headers()
		.get_all(name)
		.filter_map(|value| value.to_str().ok())
		.flat_map(|value| value.split(','))
}

/// Extracts the `for=` parameter from one RFC 7239 `Forwarded` element.
fn forwarded_for(element: &str) -> Option<IpAddr> {
	element.split(';').find_map(|pair| {
		let (name, value) = pair.trim().split_once('=')?;
		if name.eq_ignore_ascii_case("for") { parse_hop(value) } else { None }
	})
}

fn parse_hop(value: &str) -> Option<IpAddr> {
	let value = value.trim().trim_matches('"');
	value
		.parse::<IpAddr>()
		.or_else(|_| value.parse::<SocketAddr>().map(|addr| addr.ip()))
		.ok()
		.map(|ip| ip.to_canonical())
}

pub fn session_cookie(config: &Config, req: &HttpRequest, token: String) -> Cookie<'static> {
	build_cookie(token, is_https(config, req), CookieDuration::seconds(SESSION_TTL_SECS))
}

pub fn cleared_session_cookie(config: &Config, req: &HttpRequest) -> Cookie<'static> {
	build_cookie(String::new(), is_https(config, req), CookieDuration::ZERO)
}

fn build_cookie(value: String, secure: bool, max_age: CookieDuration) -> Cookie<'static> {
	Cookie::build(SESSION_COOKIE, value)
		.path("/")
		.http_only(true)
		.same_site(SameSite::Lax)
		.secure(secure)
		.max_age(max_age)
		.finish()
}

pub fn session_token(req: &HttpRequest) -> Option<String> {
	req.cookie(SESSION_COOKIE).map(|cookie| cookie.value().to_owned())
}

/// Forwarded scheme and host are honoured only from a trusted proxy; otherwise the request's own `Host` is used.
pub fn request_origin(req: &HttpRequest) -> String {
	if from_trusted_proxy(req, trusted_proxies(req)) {
		let info = req.connection_info();
		return format!("{}://{}", info.scheme(), info.host());
	}
	let scheme = if req.app_config().secure() { "https" } else { "http" };
	format!("{scheme}://{}", direct_host(req))
}

pub fn request_host(req: &HttpRequest) -> String {
	if from_trusted_proxy(req, trusted_proxies(req)) {
		return req.connection_info().host().to_owned();
	}
	direct_host(req).to_owned()
}

fn direct_host(req: &HttpRequest) -> &str {
	req.headers()
		.get(HOST)
		.and_then(|value| value.to_str().ok())
		.or_else(|| req.uri().authority().map(|authority| authority.as_str()))
		.unwrap_or_else(|| req.app_config().host())
}

pub fn normalize_public_url(value: &str) -> ApiResult<String> {
	let trimmed = value.trim().trim_end_matches('/');
	let valid = trimmed
		.parse::<Uri>()
		.is_ok_and(|uri| matches!(uri.scheme_str(), Some("http" | "https")) && uri.authority().is_some());
	if valid {
		Ok(trimmed.to_owned())
	} else {
		Err(CommonError::Invalid(format!("public URL {value} must be an absolute http(s) URL")).into())
	}
}

/// The configured public origin. Unauthenticated flows such as password reset must use this, never the Host header.
pub async fn public_url(state: &AppState) -> ApiResult<Option<String>> {
	if let Some(url) = &state.config.public_url {
		return Ok(Some(url.trim_end_matches('/').to_owned()));
	}
	Ok(state.auth.setting(PUBLIC_URL_KEY).await?)
}

pub async fn base_url(state: &AppState, req: &HttpRequest) -> ApiResult<String> {
	Ok(public_url(state).await?.unwrap_or_else(|| request_origin(req)))
}

fn is_https(config: &Config, req: &HttpRequest) -> bool {
	config
		.public_url
		.as_deref()
		.is_some_and(|url| url.starts_with("https://"))
		|| forwarded_proto_is_https(config, req)
}

fn forwarded_proto_is_https(config: &Config, req: &HttpRequest) -> bool {
	from_trusted_proxy(req, &config.trusted_proxies)
		&& req
			.headers()
			.get(X_FORWARDED_PROTO)
			.and_then(|value| value.to_str().ok())
			.and_then(|value| value.split(',').next())
			.is_some_and(|proto| proto.trim().eq_ignore_ascii_case("https"))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn ip(value: &str) -> IpAddr {
		value.parse().expect("valid address")
	}

	#[test]
	fn resolves_right_most_untrusted_hop() {
		let trusted = ["10.0.0.0/8".parse::<IpNet>().expect("valid net")];
		let peer = ip("10.0.0.1");
		let spoofed = [ip("198.51.100.9"), ip("203.0.113.7"), ip("10.0.0.5")];
		assert_eq!(resolve_client(&spoofed, &trusted, peer), ip("203.0.113.7"));
		assert_eq!(resolve_client(&[], &trusted, peer), peer);
		assert_eq!(resolve_client(&[ip("10.0.0.9"), ip("10.0.0.5")], &trusted, peer), ip("10.0.0.9"));
	}

	#[test]
	fn parses_forwarded_for_elements() {
		assert_eq!(forwarded_for("for=\"[2001:db8::17]:4711\";proto=https"), Some(ip("2001:db8::17")));
		assert_eq!(forwarded_for("For=192.0.2.43:47011"), Some(ip("192.0.2.43")));
		assert_eq!(forwarded_for("by=203.0.113.43"), None);
	}

	#[test]
	fn prefers_x_forwarded_for_over_client_sent_forwarded() {
		let req = actix_web::test::TestRequest::default()
			.insert_header((FORWARDED, "for=198.51.100.1"))
			.insert_header((X_FORWARDED_FOR, "203.0.113.7, 10.0.0.5"))
			.to_http_request();
		assert_eq!(forwarded_chain(&req), [ip("203.0.113.7"), ip("10.0.0.5")]);
		let req = actix_web::test::TestRequest::default()
			.append_header((FORWARDED, "for=192.0.2.1"))
			.append_header((FORWARDED, "for=192.0.2.2"))
			.to_http_request();
		assert_eq!(forwarded_chain(&req), [ip("192.0.2.1"), ip("192.0.2.2")]);
	}
}
