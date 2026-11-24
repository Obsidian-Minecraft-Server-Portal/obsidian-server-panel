use actix_web::{HttpResponse, web};
use obsidian_auth::{ApiTokenInfo, NewApiToken};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewTokenRequest {
	pub name: String,
	pub expires_in_days: Option<i64>,
}

#[utoipa::path(
	get,
	path = "/api/v1/account/tokens",
	tag = "account",
	responses((status = 200, description = "API tokens", body = Vec<ApiTokenInfo>))
)]
pub async fn list_api_tokens(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.list_api_tokens(user.id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/account/tokens",
	tag = "account",
	request_body = NewTokenRequest,
	responses((status = 201, description = "Token created; the secret is shown once", body = NewApiToken))
)]
pub async fn create_api_token(
	state: web::Data<AppState>,
	user: AuthUser,
	body: web::Json<NewTokenRequest>,
) -> ApiResult<HttpResponse> {
	let token = state
		.auth
		.create_api_token(user.id, &body.name, body.expires_in_days)
		.await?;
	Ok(HttpResponse::Created().json(token))
}

#[utoipa::path(
	delete,
	path = "/api/v1/account/tokens/{id}",
	tag = "account",
	params(("id" = i64, Path, description = "Token id")),
	responses((status = 204, description = "Token revoked"), (status = 404, description = "Unknown token"))
)]
pub async fn revoke_api_token(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<i64>,
) -> ApiResult<HttpResponse> {
	state.auth.revoke_api_token(user.id, id.into_inner()).await?;
	crate::ws::access_changed(&state.bus, user.id);
	Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/account/tokens")
			.route(web::get().to(list_api_tokens))
			.route(web::post().to(create_api_token)),
	)
	.service(web::resource("/account/tokens/{id}").route(web::delete().to(revoke_api_token)));
}
