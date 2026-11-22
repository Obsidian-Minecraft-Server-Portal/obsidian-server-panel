use actix_web::{HttpResponse, delete, get, patch, post, web};
use chrono::{DateTime, Utc};
use obsidian_auth::Permission;
use obsidian_common::InstanceId;
use obsidian_scheduler::{Action, Schedule, ScheduleRecord, ScheduleRun, Step, Trigger, describe, next_runs_after};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

const RUN_HISTORY_LIMIT: u32 = 50;
const PREVIEW_RUNS: usize = 5;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SchedulePatch {
	pub name: Option<String>,
	pub enabled: Option<bool>,
	pub trigger: Option<Trigger>,
	pub steps: Option<Vec<Step>>,
}

impl SchedulePatch {
	fn apply(self, schedule: &mut Schedule) {
		if let Some(name) = self.name {
			schedule.name = name;
		}
		if let Some(enabled) = self.enabled {
			schedule.enabled = enabled;
		}
		if let Some(trigger) = self.trigger {
			schedule.trigger = trigger;
		}
		if let Some(steps) = self.steps {
			schedule.steps = steps;
		}
	}
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PreviewRequest {
	pub schedule: Schedule,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PreviewResponse {
	pub description: String,
	pub next_runs: Vec<DateTime<Utc>>,
}

fn required_permissions(steps: &[Step]) -> Permission {
	steps.iter().fold(Permission::SCHEDULES_MANAGE, |required, step| {
		required
			| match step.action {
				Action::Backup => Permission::BACKUPS_CREATE,
				Action::Restart { .. } => Permission::POWER_RESTART,
				Action::Command { .. } => Permission::CONSOLE_SEND,
				Action::Start => Permission::POWER_START,
				Action::Stop => Permission::POWER_STOP,
				Action::UpdateContent => Permission::CONTENT_INSTALL | Permission::CONTENT_REMOVE,
			}
	})
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_schedules)
		.service(create_schedule)
		.service(preview_schedule)
		.service(update_schedule)
		.service(delete_schedule)
		.service(run_schedule)
		.service(schedule_runs);
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/schedules",
	tag = "schedules",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Schedules of the instance", body = Vec<ScheduleRecord>),
		(status = 403, description = "Missing schedules.manage"),
	)
)]
#[get("/instances/{id}/schedules")]
pub async fn list_schedules(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<web::Json<Vec<ScheduleRecord>>> {
	let instance = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::SCHEDULES_MANAGE).await?;
	Ok(web::Json(state.scheduler.list(instance).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/schedules",
	tag = "schedules",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = Schedule,
	responses(
		(status = 201, description = "Schedule created", body = ScheduleRecord),
		(status = 400, description = "Invalid schedule"),
		(status = 403, description = "Missing schedules.manage"),
	)
)]
#[post("/instances/{id}/schedules")]
pub async fn create_schedule(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<Schedule>,
) -> ApiResult<HttpResponse> {
	let instance = path.into_inner();
	let schedule = body.into_inner();
	require_instance_perm(&state, &user, instance, required_permissions(&schedule.steps)).await?;
	let record = state.scheduler.create(instance, schedule).await?;
	Ok(HttpResponse::Created().json(record))
}

#[utoipa::path(
	post,
	path = "/api/v1/schedules/preview",
	tag = "schedules",
	request_body = PreviewRequest,
	responses(
		(status = 200, description = "Plain-English description and the next run times", body = PreviewResponse),
		(status = 400, description = "Invalid schedule"),
		(status = 401, description = "Authentication required"),
	)
)]
#[post("/schedules/preview")]
pub async fn preview_schedule(
	state: web::Data<AppState>,
	_user: AuthUser,
	body: web::Json<PreviewRequest>,
) -> ApiResult<web::Json<PreviewResponse>> {
	let schedule = body.into_inner().schedule;
	schedule.validate()?;
	let next_runs = next_runs_after(&schedule, state.scheduler.tz(), Utc::now(), PREVIEW_RUNS)
		.into_iter()
		.map(|run| run.with_timezone(&Utc))
		.collect();
	Ok(web::Json(PreviewResponse { description: describe(&schedule), next_runs }))
}

#[utoipa::path(
	patch,
	path = "/api/v1/schedules/{id}",
	tag = "schedules",
	params(("id" = i64, Path, description = "Schedule id")),
	request_body = SchedulePatch,
	responses(
		(status = 200, description = "Schedule updated", body = ScheduleRecord),
		(status = 400, description = "Invalid schedule"),
		(status = 403, description = "Missing schedules.manage"),
		(status = 404, description = "Schedule not found"),
	)
)]
#[patch("/schedules/{id}")]
pub async fn update_schedule(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
	body: web::Json<SchedulePatch>,
) -> ApiResult<web::Json<ScheduleRecord>> {
	let id = path.into_inner();
	let current = state.scheduler.get(id).await?;
	let mut schedule = current.schedule;
	body.into_inner().apply(&mut schedule);
	require_instance_perm(&state, &user, current.instance_id, required_permissions(&schedule.steps)).await?;
	Ok(web::Json(state.scheduler.update(id, schedule).await?))
}

#[utoipa::path(
	delete,
	path = "/api/v1/schedules/{id}",
	tag = "schedules",
	params(("id" = i64, Path, description = "Schedule id")),
	responses(
		(status = 204, description = "Schedule deleted"),
		(status = 403, description = "Missing schedules.manage"),
		(status = 404, description = "Schedule not found"),
	)
)]
#[delete("/schedules/{id}")]
pub async fn delete_schedule(state: web::Data<AppState>, user: AuthUser, path: web::Path<i64>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let current = state.scheduler.get(id).await?;
	require_instance_perm(&state, &user, current.instance_id, Permission::SCHEDULES_MANAGE).await?;
	state.scheduler.delete(id).await?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/schedules/{id}/run",
	tag = "schedules",
	params(("id" = i64, Path, description = "Schedule id")),
	responses(
		(status = 202, description = "Run started in the background"),
		(status = 403, description = "Missing schedules.manage or a permission a step needs"),
		(status = 409, description = "The schedule is already running"),
		(status = 404, description = "Schedule not found"),
	)
)]
#[post("/schedules/{id}/run")]
pub async fn run_schedule(state: web::Data<AppState>, user: AuthUser, path: web::Path<i64>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let current = state.scheduler.get(id).await?;
	require_instance_perm(&state, &user, current.instance_id, required_permissions(&current.schedule.steps)).await?;
	state.scheduler.run_now(id).await?;
	Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
	get,
	path = "/api/v1/schedules/{id}/runs",
	tag = "schedules",
	params(("id" = i64, Path, description = "Schedule id")),
	responses(
		(status = 200, description = "Most recent runs, newest first", body = Vec<ScheduleRun>),
		(status = 403, description = "Missing schedules.manage"),
		(status = 404, description = "Schedule not found"),
	)
)]
#[get("/schedules/{id}/runs")]
pub async fn schedule_runs(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<i64>,
) -> ApiResult<web::Json<Vec<ScheduleRun>>> {
	let id = path.into_inner();
	let current = state.scheduler.get(id).await?;
	require_instance_perm(&state, &user, current.instance_id, Permission::SCHEDULES_MANAGE).await?;
	Ok(web::Json(state.scheduler.runs(id, RUN_HISTORY_LIMIT).await?))
}
