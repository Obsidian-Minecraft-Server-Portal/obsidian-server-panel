use std::str::FromStr;

use actix_web::{HttpResponse, get, web};
use obsidian_common::{CommonError, Loader};
use obsidian_loaders::{LOADER_KINDS, loader};

use crate::auth::AuthUser;
use crate::error::ApiResult;

#[utoipa::path(
	get,
	path = "/api/v1/loaders",
	tag = "loaders",
	responses((status = 200, description = "Supported server loader kinds", body = Vec<String>))
)]
#[get("/loaders")]
pub async fn list_loaders(_user: AuthUser) -> ApiResult<HttpResponse> {
	let kinds: Vec<String> = LOADER_KINDS.iter().map(ToString::to_string).collect();
	Ok(HttpResponse::Ok().json(kinds))
}

#[utoipa::path(
	get,
	path = "/api/v1/loaders/{kind}/versions",
	tag = "loaders",
	params(("kind" = String, Path, description = "Loader kind, e.g. fabric")),
	responses(
		(status = 200, description = "Minecraft versions supported by the loader", body = Vec<String>),
		(status = 400, description = "Unknown loader kind")
	)
)]
#[get("/loaders/{kind}/versions")]
pub async fn minecraft_versions(_user: AuthUser, kind: web::Path<String>) -> ApiResult<HttpResponse> {
	let versions = loader(parse_kind(&kind)?).mc_versions().await?;
	Ok(HttpResponse::Ok().json(versions))
}

#[utoipa::path(
	get,
	path = "/api/v1/loaders/{kind}/versions/{mc}",
	tag = "loaders",
	params(
		("kind" = String, Path, description = "Loader kind, e.g. fabric"),
		("mc" = String, Path, description = "Minecraft version")
	),
	responses(
		(status = 200, description = "Loader versions for the Minecraft version", body = Vec<String>),
		(status = 400, description = "Unknown loader kind or invalid version")
	)
)]
#[get("/loaders/{kind}/versions/{mc}")]
pub async fn loader_versions(_user: AuthUser, path: web::Path<(String, String)>) -> ApiResult<HttpResponse> {
	let (kind, mc) = path.into_inner();
	let versions = loader(parse_kind(&kind)?).loader_versions(&mc).await?;
	Ok(HttpResponse::Ok().json(versions))
}

fn parse_kind(kind: &str) -> ApiResult<Loader> {
	Loader::from_str(kind).map_err(|_| CommonError::Invalid(format!("unknown loader '{kind}'")).into())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_loaders).service(minecraft_versions).service(loader_versions);
}
