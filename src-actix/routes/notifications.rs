use actix_web::{HttpResponse, web};
use obsidian_auth::Notification;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkReadRequest {
	/// Notification ids to mark read. Omit or null to mark every unread notification.
	pub ids: Option<Vec<i64>>,
}

#[utoipa::path(
	get,
	path = "/api/v1/notifications",
	tag = "notifications",
	responses((status = 200, description = "Recent notifications, newest first", body = Vec<Notification>))
)]
pub async fn list_notifications(state: web::Data<AppState>, user: AuthUser) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.auth.list_notifications(user.id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/notifications/read",
	tag = "notifications",
	request_body = MarkReadRequest,
	responses((status = 204, description = "Marked read"))
)]
pub async fn mark_notifications_read(
	state: web::Data<AppState>,
	user: AuthUser,
	body: web::Json<MarkReadRequest>,
) -> ApiResult<HttpResponse> {
	state.auth.mark_notifications_read(user.id, body.ids.as_deref()).await?;
	Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("/notifications").route(web::get().to(list_notifications)))
		.service(web::resource("/notifications/read").route(web::post().to(mark_notifications_read)));
}
