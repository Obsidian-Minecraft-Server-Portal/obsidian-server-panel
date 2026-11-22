use std::sync::Arc;

use actix_web::{HttpResponse, delete, get, post, web};
use obsidian_content::{AnalysisResult, AnalyzeRequest, ExportRequest};
use serde_json::json;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(analyze_modpack)
		.service(get_draft)
		.service(draft_icon)
		.service(discard_draft)
		.service(export_draft);
}

#[utoipa::path(
	post,
	path = "/api/v1/modpacks/analyze",
	tag = "content",
	request_body = AnalyzeRequest,
	responses(
		(status = 202, description = "Analysis started; returns {task_id} and progress is published on the tasks topic"),
		(status = 401, description = "Authentication required"),
	)
)]
#[post("/modpacks/analyze")]
pub async fn analyze_modpack(
	state: web::Data<AppState>,
	user: AuthUser,
	web::Json(request): web::Json<AnalyzeRequest>,
) -> ApiResult<HttpResponse> {
	let task_id = state.content.start_analysis(user.id, request)?;
	Ok(HttpResponse::Accepted().json(json!({ "task_id": task_id })))
}

#[utoipa::path(
	get,
	path = "/api/v1/modpacks/drafts/{id}",
	tag = "content",
	responses(
		(status = 200, description = "Analysis of a modpack for the server pack builder", body = AnalysisResult),
		(status = 404, description = "The draft does not exist or has expired"),
	)
)]
#[get("/modpacks/drafts/{id}")]
pub async fn get_draft(state: web::Data<AppState>, user: AuthUser, path: web::Path<String>) -> ApiResult<HttpResponse> {
	let draft = state.content.draft(user.id, &path.into_inner())?;
	Ok(HttpResponse::Ok().json(draft))
}

#[utoipa::path(
	get,
	path = "/api/v1/modpacks/drafts/{id}/mods/{index}/icon",
	tag = "content",
	responses(
		(status = 200, description = "Mod icon as PNG", content_type = "image/png"),
		(status = 404, description = "The mod has no icon"),
	)
)]
#[get("/modpacks/drafts/{id}/mods/{index}/icon")]
pub async fn draft_icon(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(String, usize)>,
) -> ApiResult<HttpResponse> {
	let (id, index) = path.into_inner();
	let bytes = state.content.draft_icon(user.id, &id, index).await?;
	Ok(HttpResponse::Ok().content_type("image/png").body(bytes))
}

#[utoipa::path(
	delete,
	path = "/api/v1/modpacks/drafts/{id}",
	tag = "content",
	responses(
		(status = 204, description = "Draft discarded and its downloads removed"),
		(status = 404, description = "The draft does not exist"),
	)
)]
#[delete("/modpacks/drafts/{id}")]
pub async fn discard_draft(state: web::Data<AppState>, user: AuthUser, path: web::Path<String>) -> ApiResult<HttpResponse> {
	state.content.discard_draft(user.id, &path.into_inner())?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/modpacks/drafts/{id}/export",
	tag = "content",
	request_body = ExportRequest,
	responses(
		(status = 202, description = "Export started; returns {task_id} and progress is published on the tasks topic"),
		(status = 404, description = "The draft does not exist"),
	)
)]
#[post("/modpacks/drafts/{id}/export")]
pub async fn export_draft(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<String>,
	web::Json(request): web::Json<ExportRequest>,
) -> ApiResult<HttpResponse> {
	let task_id = state
		.content
		.start_export(Arc::clone(&state.instances), user.id, &path.into_inner(), request)?;
	Ok(HttpResponse::Accepted().json(json!({ "task_id": task_id })))
}
