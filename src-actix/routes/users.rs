use actix_web::{HttpResponse, web};
use obsidian_auth::{NewUser, UserProfile, UserUpdate};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::auth::AdminUser;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetPasswordRequest {
	pub password: String,
}

#[utoipa::path(
	get,
	path = "/api/v1/users",
	tag = "users",
	responses((status = 200, description = "All accounts", body = Vec<UserProfile>), (status = 403, description = "Administrator only"))
)]
pub async fn list_users(state: web::Data<AppState>, _admin: AdminUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.list_users().await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/users",
	tag = "users",
	request_body = NewUser,
	responses((status = 201, description = "Account created", body = UserProfile), (status = 409, description = "Username taken"))
)]
pub async fn create_user(
	state: web::Data<AppState>,
	admin: AdminUser,
	body: web::Json<NewUser>,
) -> ApiResult<HttpResponse> {
	let profile = state.auth.create_user(admin.id, &body).await?;
	Ok(HttpResponse::Created().json(profile))
}

#[utoipa::path(
	get,
	path = "/api/v1/users/{id}",
	tag = "users",
	params(("id" = i64, Path, description = "User id")),
	responses((status = 200, description = "Account", body = UserProfile), (status = 404, description = "Unknown user"))
)]
pub async fn get_user(state: web::Data<AppState>, _admin: AdminUser, id: web::Path<i64>) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.profile(id.into_inner()).await?))
}

#[utoipa::path(
	patch,
	path = "/api/v1/users/{id}",
	tag = "users",
	params(("id" = i64, Path, description = "User id")),
	request_body = UserUpdate,
	responses((status = 200, description = "Updated account", body = UserProfile), (status = 409, description = "Would remove the last administrator"))
)]
pub async fn update_user(
	state: web::Data<AppState>,
	admin: AdminUser,
	id: web::Path<i64>,
	body: web::Json<UserUpdate>,
) -> ApiResult<HttpResponse> {
	let profile = state.auth.update_user(admin.id, id.into_inner(), &body).await?;
	Ok(HttpResponse::Ok().json(profile))
}

#[utoipa::path(
	delete,
	path = "/api/v1/users/{id}",
	tag = "users",
	params(("id" = i64, Path, description = "User id")),
	responses((status = 204, description = "Account deleted"), (status = 409, description = "User owns instances or is the last administrator"))
)]
pub async fn delete_user(state: web::Data<AppState>, admin: AdminUser, id: web::Path<i64>) -> ApiResult<HttpResponse> {
	state.auth.delete_user(admin.id, id.into_inner()).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/users/{id}/password",
	tag = "users",
	params(("id" = i64, Path, description = "User id")),
	request_body = SetPasswordRequest,
	responses((status = 204, description = "Password replaced; the user's sessions were signed out"))
)]
pub async fn set_password(
	state: web::Data<AppState>,
	admin: AdminUser,
	id: web::Path<i64>,
	body: web::Json<SetPasswordRequest>,
) -> ApiResult<HttpResponse> {
	state
		.auth
		.set_user_password(admin.id, id.into_inner(), &body.password)
		.await?;
	Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/users")
			.route(web::get().to(list_users))
			.route(web::post().to(create_user)),
	)
	.service(
		web::resource("/users/{id}")
			.route(web::get().to(get_user))
			.route(web::patch().to(update_user))
			.route(web::delete().to(delete_user)),
	)
	.service(web::resource("/users/{id}/password").route(web::post().to(set_password)));
}
