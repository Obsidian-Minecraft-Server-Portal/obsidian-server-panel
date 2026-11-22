use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{AUTHORIZATION, HeaderValue, ORIGIN, REFERER, UPGRADE};
use actix_web::http::uri::Authority;
use actix_web::http::{Method, Uri};
use actix_web::middleware::Next;
use actix_web::{Error, web};
use obsidian_common::CommonError;

use crate::error::ApiError;
use crate::routes::auth::support::request_host;
use crate::state::AppState;

pub async fn verify_origin(
	req: ServiceRequest,
	next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
	if needs_check(&req) && !trusted_origin(&req) {
		return Err(ApiError::from(CommonError::Forbidden("cross-origin request rejected".into())).into());
	}
	next.call(req).await
}

fn needs_check(req: &ServiceRequest) -> bool {
	let unsafe_method = !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
	(unsafe_method || req.headers().contains_key(UPGRADE)) && !req.headers().contains_key(AUTHORIZATION)
}

fn trusted_origin(req: &ServiceRequest) -> bool {
	let headers = req.headers();
	let Some(source) = headers.get(ORIGIN).or_else(|| headers.get(REFERER)) else {
		return true;
	};
	let Some(source) = authority(source) else {
		return false;
	};
	if source.as_str().eq_ignore_ascii_case(&request_host(req.request())) {
		return true;
	}
	req.app_data::<web::Data<AppState>>()
		.and_then(|state| state.config.public_url.as_deref())
		.and_then(|url| url.parse::<Uri>().ok())
		.and_then(|uri| uri.authority().map(|a| a.as_str().eq_ignore_ascii_case(source.as_str())))
		.unwrap_or(false)
}

fn authority(value: &HeaderValue) -> Option<Authority> {
	value.to_str().ok()?.parse::<Uri>().ok()?.into_parts().authority
}
