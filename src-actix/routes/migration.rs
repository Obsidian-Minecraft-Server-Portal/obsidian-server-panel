use actix_web::{HttpResponse, get, post, web};
use obsidian_auth::Permission;
use obsidian_common::InstanceId;
use obsidian_migration::{MigrationRecord, ScanReport, ScanRequest, StartRequest, StartedMigration};
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct ScanResponse {
	pub report: ScanReport,
	pub java_component: Option<String>,
	pub java_major_version: Option<u8>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RollbackStarted {
	pub task_id: u64,
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/migration/scan",
	tag = "migration",
	request_body = ScanRequest,
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Compatibility of each mod and plugin with the target version", body = ScanResponse),
		(status = 403, description = "Missing settings.edit"),
		(status = 409, description = "Another migration is running"),
	)
)]
#[post("/instances/{id}/migration/scan")]
pub async fn scan(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<ScanRequest>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	let report = state.migrations.scan(id, &body.into_inner()).await?;
	let java = state.java.requirement(&report.target.mc_version).await.ok();
	Ok(HttpResponse::Ok().json(ScanResponse {
		report,
		java_component: java.as_ref().map(|requirement| requirement.component.clone()),
		java_major_version: java.map(|requirement| requirement.major_version),
	}))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/migration",
	tag = "migration",
	request_body = StartRequest,
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 202, description = "Migration started as an 'instance.migrate' task", body = StartedMigration),
		(status = 400, description = "A mod has no decision, or the target is not valid"),
		(status = 403, description = "Missing settings.edit, content.install, content.remove or backups.create"),
		(status = 409, description = "The server is running, installing or already migrating"),
	)
)]
#[post("/instances/{id}/migration")]
pub async fn start(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<StartRequest>,
) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	let request = body.into_inner();
	require_instance_perm(
		&state,
		&user,
		id,
		Permission::SETTINGS_EDIT | Permission::CONTENT_INSTALL | Permission::CONTENT_REMOVE,
	)
	.await?;
	if request.backup {
		require_instance_perm(&state, &user, id, Permission::BACKUPS_CREATE).await?;
	}
	let started = state.migrations.start(id, request, user.id).await?;
	Ok(HttpResponse::Accepted().json(started))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/migrations",
	tag = "migration",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Migrations, newest first", body = Vec<MigrationRecord>),
		(status = 403, description = "Missing settings.edit"),
	)
)]
#[get("/instances/{id}/migrations")]
pub async fn history(state: web::Data<AppState>, user: AuthUser, path: web::Path<InstanceId>) -> ApiResult<HttpResponse> {
	let id = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT).await?;
	Ok(HttpResponse::Ok().json(state.migrations.history(id).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/migrations/{migration_id}/rollback",
	tag = "migration",
	params(
		("id" = i64, Path, description = "Instance id"),
		("migration_id" = i64, Path, description = "Migration id"),
	),
	responses(
		(status = 202, description = "Rollback started as an 'instance.rollback' task", body = RollbackStarted),
		(status = 403, description = "Missing settings.edit or backups.restore"),
		(status = 409, description = "The migration has no backup, is still running, or was already rolled back"),
	)
)]
#[post("/instances/{id}/migrations/{migration_id}/rollback")]
pub async fn rollback(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, i64)>,
) -> ApiResult<HttpResponse> {
	let (id, migration_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::SETTINGS_EDIT | Permission::BACKUPS_RESTORE).await?;
	let task_id = state.migrations.rollback(id, migration_id, user.id).await?;
	Ok(HttpResponse::Accepted().json(RollbackStarted { task_id }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(scan).service(start).service(history).service(rollback);
}
