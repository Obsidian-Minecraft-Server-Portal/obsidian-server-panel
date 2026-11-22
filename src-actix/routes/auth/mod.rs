pub mod login;
pub mod recovery;
pub mod support;

use std::net::IpAddr;

use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::{Governor, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;
use actix_web::web;
use ipnet::Ipv6Net;
use support::client_ip;

const LOGIN_SECS_PER_REQUEST: u64 = 6;
const LOGIN_BURST: u32 = 10;
const RECOVERY_SECS_PER_REQUEST: u64 = 60;
const RECOVERY_BURST: u32 = 5;
const IPV6_BUCKET_PREFIX: u8 = 64;

/// Per-client-IP limiter for endpoints that accept unauthenticated guesses. Honours trusted reverse proxies.
pub fn rate_limit(seconds_per_request: u64, burst: u32) -> Governor<ClientIpKeyExtractor, NoOpMiddleware> {
	let config = GovernorConfigBuilder::default()
		.seconds_per_request(seconds_per_request)
		.burst_size(burst)
		.key_extractor(ClientIpKeyExtractor)
		.finish()
		.expect("rate limit configuration is valid");
	Governor::new(&config)
}

#[derive(Clone)]
pub struct ClientIpKeyExtractor;

impl KeyExtractor for ClientIpKeyExtractor {
	type Key = IpAddr;
	type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

	fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
		client_ip(req.request())
			.map(bucket)
			.ok_or_else(|| SimpleKeyExtractionError::new("could not determine the client address"))
	}
}

/// IPv6 clients usually own a whole /64, so per-address buckets would be trivial to rotate around.
fn bucket(ip: IpAddr) -> IpAddr {
	match ip {
		IpAddr::V6(v6) => Ipv6Net::new(v6, IPV6_BUCKET_PREFIX).map_or(ip, |net| net.network().into()),
		IpAddr::V4(_) => ip,
	}
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/auth/login")
			.wrap(rate_limit(LOGIN_SECS_PER_REQUEST, LOGIN_BURST))
			.route(web::post().to(login::login)),
	)
	.service(web::resource("/auth/logout").route(web::post().to(login::logout)))
	.service(web::resource("/auth/me").route(web::get().to(login::me)))
	.service(
		web::resource("/auth/forgot-password")
			.wrap(rate_limit(RECOVERY_SECS_PER_REQUEST, RECOVERY_BURST))
			.route(web::post().to(recovery::forgot_password)),
	)
	.service(web::resource("/auth/reset-password").route(web::post().to(recovery::reset_password)));
}
