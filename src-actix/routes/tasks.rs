use actix_web::{HttpResponse, delete, get, web};
use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, TaskRecord, TaskStatus, UserId};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskView {
	pub id: u64,
	pub kind: String,
	pub title: String,
	pub owner: Option<UserId>,
	#[schema(value_type = String)]
	pub status: TaskStatus,
	pub progress: f32,
	pub message: String,
	#[schema(value_type = Option<Object>)]
	pub result: Option<Value>,
	pub error: Option<String>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl From<TaskRecord> for TaskView {
	fn from(task: TaskRecord) -> Self {
		Self {
			id: task.id,
			kind: task.kind,
			title: task.title,
			owner: task.owner,
			status: task.status,
			progress: task.progress,
			message: task.message,
			result: task.result,
			error: task.error,
			created_at: task.created_at,
			updated_at: task.updated_at,
		}
	}
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_tasks).service(dismiss_task);
}

#[utoipa::path(
	get,
	path = "/api/v1/tasks",
	tag = "tasks",
	responses(
		(status = 200, description = "Global tasks and tasks owned by the caller, or all tasks for admins", body = Vec<TaskView>),
		(status = 401, description = "Authentication required"),
	)
)]
#[get("/tasks")]
pub async fn list_tasks(state: web::Data<AppState>, user: AuthUser) -> ApiResult<web::Json<Vec<TaskView>>> {
	let tasks = if user.is_admin { state.tasks.list() } else { state.tasks.list_for(user.id) };
	Ok(web::Json(tasks.into_iter().map(TaskView::from).collect()))
}

#[utoipa::path(
	delete,
	path = "/api/v1/tasks/{id}",
	tag = "tasks",
	params(("id" = u64, Path, description = "Task id")),
	responses(
		(status = 204, description = "Finished task dismissed"),
		(status = 401, description = "Authentication required"),
		(status = 403, description = "Global tasks can only be dismissed by administrators"),
		(status = 404, description = "Task not found or not visible to the caller"),
		(status = 409, description = "Task is still running"),
	)
)]
#[delete("/tasks/{id}")]
pub async fn dismiss_task(state: web::Data<AppState>, user: AuthUser, path: web::Path<u64>) -> ApiResult<HttpResponse> {
	let task = state
		.tasks
		.get(path.into_inner())
		.filter(|task| user.is_admin || task.owner.is_none_or(|owner| owner == user.id))
		.ok_or_else(|| CommonError::NotFound("task".into()))?;
	if task.owner.is_none() && !user.is_admin {
		return Err(CommonError::Forbidden("only administrators can dismiss global tasks".into()).into());
	}
	if task.status == TaskStatus::Running {
		return Err(CommonError::Conflict("task is still running".into()).into());
	}
	state.tasks.remove(task.id);
	Ok(HttpResponse::NoContent().finish())
}
