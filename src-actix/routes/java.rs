use std::sync::Arc;

use actix_web::{HttpResponse, delete, get, post, web};
use obsidian_common::CommonError;
use obsidian_java::{AvailableJava, JavaRequirement, JavaRuntime, is_component};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::{AdminUser, AuthUser};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct RequirementQuery {
	pub mc_version: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InstallJavaRequest {
	pub component: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskStarted {
	pub task_id: u64,
}

#[utoipa::path(
	get,
	path = "/api/v1/java",
	tag = "java",
	responses((status = 200, description = "Installed and detected Java runtimes", body = Vec<JavaRuntime>))
)]
#[get("/java")]
pub async fn list_runtimes(_user: AuthUser, state: web::Data<AppState>) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.java.list().await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/java/requirement",
	tag = "java",
	params(RequirementQuery),
	responses((status = 200, description = "Java runtime a Minecraft version needs", body = JavaRequirement))
)]
#[get("/java/requirement")]
pub async fn requirement(
	_user: AuthUser,
	state: web::Data<AppState>,
	query: web::Query<RequirementQuery>,
) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.java.requirement(&query.mc_version).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/java/available",
	tag = "java",
	responses((status = 200, description = "Java runtimes Mojang publishes for this platform", body = Vec<AvailableJava>))
)]
#[get("/java/available")]
pub async fn available_runtimes(_user: AuthUser, state: web::Data<AppState>) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.java.available().await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/java/install",
	tag = "java",
	request_body = InstallJavaRequest,
	responses(
		(status = 202, description = "Install task started", body = TaskStarted),
		(status = 400, description = "Unknown component")
	)
)]
#[post("/java/install")]
pub async fn install_runtime(
	admin: AdminUser,
	state: web::Data<AppState>,
	body: web::Json<InstallJavaRequest>,
) -> ApiResult<HttpResponse> {
	let component = body.into_inner().component;
	if !is_component(&component) {
		return Err(CommonError::Invalid(format!("unknown java component '{component}'")).into());
	}
	let task = state.tasks.create("java_install", format!("Install {component}"), Some(admin.id));
	let started = TaskStarted { task_id: task.id() };
	let java = Arc::clone(&state.java);
	tokio::spawn(async move {
		match java.install(&component, &task).await {
			Ok(runtime) => task.finish(serde_json::json!(runtime)),
			Err(error) => task.fail(error),
		}
	});
	Ok(HttpResponse::Accepted().json(started))
}

#[utoipa::path(
	post,
	path = "/api/v1/java/detect",
	tag = "java",
	responses((status = 200, description = "Java runtimes found on this system", body = Vec<JavaRuntime>))
)]
#[post("/java/detect")]
pub async fn detect_runtimes(_admin: AdminUser, state: web::Data<AppState>) -> ApiResult<HttpResponse> {
	Ok(HttpResponse::Ok().json(state.java.detect_system().await?))
}

#[utoipa::path(
	delete,
	path = "/api/v1/java/{id}",
	tag = "java",
	params(("id" = i64, Path, description = "Java runtime id")),
	responses(
		(status = 204, description = "Runtime removed"),
		(status = 409, description = "Runtime is used by an instance")
	)
)]
#[delete("/java/{id}")]
pub async fn remove_runtime(
	_admin: AdminUser,
	state: web::Data<AppState>,
	id: web::Path<i64>,
) -> ApiResult<HttpResponse> {
	state.java.remove(id.into_inner()).await?;
	Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_runtimes)
		.service(requirement)
		.service(available_runtimes)
		.service(install_runtime)
		.service(detect_runtimes)
		.service(remove_runtime);
}
