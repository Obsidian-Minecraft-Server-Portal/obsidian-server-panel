use actix_web::{HttpRequest, HttpResponse, web};
use obsidian_auth::{Invite, NewUser, UserProfile};
use obsidian_common::CommonError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::AdminUser;
use crate::error::ApiResult;
use crate::routes::auth::support::{base_url, client, session_cookie};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
	pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedInvite {
	pub invite: Invite,
	/// Shown once; only its hash is stored.
	pub token: String,
	pub url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteStatus {
	pub valid: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInviteRequest {
	pub username: String,
	pub email: Option<String>,
	pub password: String,
}

#[utoipa::path(
	get,
	path = "/api/v1/invites",
	tag = "invites",
	responses((status = 200, description = "Recent invites", body = Vec<Invite>))
)]
pub async fn list_invites(state: web::Data<AppState>, _admin: AdminUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.list_invites().await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/invites",
	tag = "invites",
	request_body = CreateInviteRequest,
	responses((status = 201, description = "Invite created", body = CreatedInvite))
)]
pub async fn create_invite(
	state: web::Data<AppState>,
	req: HttpRequest,
	admin: AdminUser,
	body: web::Json<CreateInviteRequest>,
) -> ApiResult<HttpResponse> {
	let (invite, token) = state.auth.create_invite(admin.id, body.expires_in_hours).await?;
	let url = format!("{}/invite/{token}", base_url(&state, &req).await?);
	Ok(HttpResponse::Created().json(CreatedInvite { invite, token, url }))
}

#[utoipa::path(
	get,
	path = "/api/v1/invites/{key}",
	tag = "invites",
	params(("key" = String, Path, description = "Invite token")),
	responses((status = 200, description = "Whether the invite can still be accepted", body = InviteStatus))
)]
pub async fn check_invite(state: web::Data<AppState>, key: web::Path<String>) -> ApiResult<HttpResponse> {
	let valid = state.auth.invite_is_valid(&key).await?;
	Ok(HttpResponse::Ok().json(InviteStatus { valid }))
}

#[utoipa::path(
	delete,
	path = "/api/v1/invites/{key}",
	tag = "invites",
	params(("key" = i64, Path, description = "Invite id")),
	responses((status = 204, description = "Unused invite revoked"), (status = 404, description = "Unknown or already used invite"))
)]
pub async fn revoke_invite(
	state: web::Data<AppState>,
	admin: AdminUser,
	key: web::Path<String>,
) -> ApiResult<HttpResponse> {
	let id: i64 = key.parse().map_err(|_| CommonError::NotFound("invite".into()))?;
	state.auth.revoke_invite(admin.id, id).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/invites/{token}/accept",
	tag = "invites",
	params(("token" = String, Path, description = "Invite token")),
	request_body = AcceptInviteRequest,
	responses(
		(status = 201, description = "Account created and signed in", body = UserProfile),
		(status = 400, description = "Invite is invalid or expired, or details are invalid"),
		(status = 409, description = "Username taken")
	)
)]
pub async fn accept_invite(
	state: web::Data<AppState>,
	req: HttpRequest,
	token: web::Path<String>,
	body: web::Json<AcceptInviteRequest>,
) -> ApiResult<HttpResponse> {
	let body = body.into_inner();
	let user = NewUser {
		username: body.username,
		email: body.email.filter(|email| !email.is_empty()),
		password: body.password,
		is_admin: false,
		instance_limit: None,
	};
	let (session, profile) = state.auth.accept_invite(&token, &user, &client(&req)).await?;
	Ok(HttpResponse::Created()
		.cookie(session_cookie(&state.config, &req, session))
		.json(profile))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/invites")
			.route(web::get().to(list_invites))
			.route(web::post().to(create_invite)),
	)
	.service(
		web::resource("/invites/{key}")
			.route(web::get().to(check_invite))
			.route(web::delete().to(revoke_invite)),
	)
	.service(web::resource("/invites/{token}/accept").route(web::post().to(accept_invite)));
}
