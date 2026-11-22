use std::sync::Arc;

use actix_web::{HttpResponse, post, web};
use obsidian_content::ModpackRequest;
use serde_json::json;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(install_modpack);
}

#[utoipa::path(
	post,
	path = "/api/v1/modpacks/install",
	tag = "content",
	request_body = ModpackRequest,
	responses(
		(status = 202, description = "Modpack install started; returns {task_id} and progress is published on the tasks topic"),
		(status = 401, description = "Authentication required"),
	)
)]
#[post("/modpacks/install")]
pub async fn install_modpack(
	state: web::Data<AppState>,
	user: AuthUser,
	web::Json(request): web::Json<ModpackRequest>,
) -> ApiResult<HttpResponse> {
	let task_id = state
		.content
		.start_modpack(Arc::clone(&state.instances), user.id, request)?;
	Ok(HttpResponse::Accepted().json(json!({ "task_id": task_id })))
}
