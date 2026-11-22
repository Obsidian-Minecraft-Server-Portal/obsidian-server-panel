use std::io::{self, Write};

use actix_web::http::header::CONTENT_DISPOSITION;
use actix_web::web::Bytes;
use actix_web::{HttpResponse, delete, get, post, put, web};
use futures_util::stream;
use obsidian_auth::Permission;
use obsidian_backups::{BackupPolicy, SnapshotInfo, StoreStats, TreeEntry};
use obsidian_common::{CommonError, InstanceId};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use utoipa::ToSchema;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::schedule_executor::{MANUAL_LABEL, backup_now, is_live, restore_now};
use crate::state::AppState;

const DOWNLOAD_QUEUE: usize = 16;

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupStarted {
	pub task_id: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RestoreRequest {
	#[serde(default)]
	pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TreeQuery {
	#[serde(default)]
	pub path: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(list_backups)
		.service(create_backup)
		.service(backup_stats)
		.service(get_policy)
		.service(set_policy)
		.service(backup_tree)
		.service(restore_backup)
		.service(download_backup)
		.service(delete_backup);
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/backups",
	tag = "backups",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Snapshots, newest first", body = Vec<SnapshotInfo>),
		(status = 403, description = "Missing backups.download"),
		(status = 404, description = "Instance not found"),
	)
)]
#[get("/instances/{id}/backups")]
pub async fn list_backups(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<web::Json<Vec<SnapshotInfo>>> {
	let instance = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_DOWNLOAD).await?;
	Ok(web::Json(state.backups.list(instance).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/backups",
	tag = "backups",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 202, description = "Snapshot started as a background task", body = BackupStarted),
		(status = 403, description = "Missing backups.create"),
		(status = 404, description = "Instance not found"),
	)
)]
#[post("/instances/{id}/backups")]
pub async fn create_backup(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<HttpResponse> {
	let instance = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_CREATE).await?;
	let name = state.instances.get(instance).await?.name;
	let task = state.tasks.create("backup", format!("Backup of {name}"), Some(user.id));
	let task_id = task.id();
	let (instances, backups, bus) = (state.instances.clone(), state.backups.clone(), state.bus.clone());
	tokio::spawn(async move {
		let _ = backup_now(&instances, &backups, &bus, &task, instance, MANUAL_LABEL).await;
	});
	Ok(HttpResponse::Accepted().json(BackupStarted { task_id }))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/backups/stats",
	tag = "backups",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Snapshot count, logical size, stored size and dedup ratio", body = StoreStats),
		(status = 403, description = "Missing backups.download"),
	)
)]
#[get("/instances/{id}/backups/stats")]
pub async fn backup_stats(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<web::Json<StoreStats>> {
	let instance = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_DOWNLOAD).await?;
	Ok(web::Json(state.backups.stats(instance).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/backup-policy",
	tag = "backups",
	params(("id" = i64, Path, description = "Instance id")),
	responses(
		(status = 200, description = "Retention and exclude policy", body = BackupPolicy),
		(status = 403, description = "Missing backups.download"),
	)
)]
#[get("/instances/{id}/backup-policy")]
pub async fn get_policy(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
) -> ApiResult<web::Json<BackupPolicy>> {
	let instance = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_DOWNLOAD).await?;
	Ok(web::Json(state.backups.policy(instance).await?))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/backup-policy",
	tag = "backups",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = BackupPolicy,
	responses(
		(status = 200, description = "Policy saved", body = BackupPolicy),
		(status = 400, description = "Invalid exclude pattern"),
		(status = 403, description = "Missing backups.create"),
	)
)]
#[put("/instances/{id}/backup-policy")]
pub async fn set_policy(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<InstanceId>,
	body: web::Json<BackupPolicy>,
) -> ApiResult<web::Json<BackupPolicy>> {
	let instance = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_CREATE).await?;
	state.backups.set_policy(instance, &body).await?;
	Ok(web::Json(state.backups.policy(instance).await?))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/backups/{snap}/tree",
	tag = "backups",
	params(
		("id" = i64, Path, description = "Instance id"),
		("snap" = String, Path, description = "Snapshot id"),
		("path" = Option<String>, Query, description = "Directory inside the snapshot; empty for the root"),
	),
	responses(
		(status = 200, description = "Direct children of the directory", body = Vec<TreeEntry>),
		(status = 403, description = "Missing backups.download"),
		(status = 404, description = "Snapshot not found"),
	)
)]
#[get("/instances/{id}/backups/{snap}/tree")]
pub async fn backup_tree(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
	query: web::Query<TreeQuery>,
) -> ApiResult<web::Json<Vec<TreeEntry>>> {
	let (instance, snap) = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_DOWNLOAD).await?;
	Ok(web::Json(state.backups.tree(instance, &snap, &query.path).await?))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/backups/{snap}/restore",
	tag = "backups",
	params(
		("id" = i64, Path, description = "Instance id"),
		("snap" = String, Path, description = "Snapshot id"),
	),
	request_body = Option<RestoreRequest>,
	responses(
		(status = 202, description = "Restore started as a background task", body = BackupStarted),
		(status = 403, description = "Missing backups.restore"),
		(status = 404, description = "Snapshot not found"),
		(status = 409, description = "The server is running"),
	)
)]
#[post("/instances/{id}/backups/{snap}/restore")]
pub async fn restore_backup(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
	body: Option<web::Json<RestoreRequest>>,
) -> ApiResult<HttpResponse> {
	let (instance, snap) = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_RESTORE).await?;
	if is_live(&state.instances, instance).await? {
		return Err(CommonError::Conflict("stop the server before restoring a backup".into()).into());
	}
	state.backups.get(instance, &snap).await?;
	let name = state.instances.get(instance).await?.name;
	let task = state.tasks.create("backup_restore", format!("Restore backup of {name}"), Some(user.id));
	let task_id = task.id();
	let subpath = body.and_then(|body| body.into_inner().path);
	let (instances, backups, bus) = (state.instances.clone(), state.backups.clone(), state.bus.clone());
	tokio::spawn(async move {
		let _ = restore_now(&instances, &backups, &bus, &task, instance, &snap, subpath).await;
	});
	Ok(HttpResponse::Accepted().json(BackupStarted { task_id }))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/backups/{snap}/download",
	tag = "backups",
	params(
		("id" = i64, Path, description = "Instance id"),
		("snap" = String, Path, description = "Snapshot id"),
	),
	responses(
		(status = 200, description = "Snapshot as a streamed zip archive", content_type = "application/zip"),
		(status = 403, description = "Missing backups.download"),
		(status = 404, description = "Snapshot not found"),
	)
)]
#[get("/instances/{id}/backups/{snap}/download")]
pub async fn download_backup(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
) -> ApiResult<HttpResponse> {
	let (instance, snap) = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_DOWNLOAD).await?;
	let info = state.backups.get(instance, &snap).await?;
	let (sender, receiver) = mpsc::channel::<Bytes>(DOWNLOAD_QUEUE);
	let backups = state.backups.clone();
	let export_id = snap.clone();
	tokio::spawn(async move {
		if let Err(error) = backups.export_zip(instance, &export_id, ChannelWriter(sender)).await {
			tracing::error!(%error, instance, "backup download aborted");
		}
	});
	let body = stream::unfold(receiver, |mut receiver| async move {
		receiver.recv().await.map(|chunk| (Ok::<Bytes, actix_web::Error>(chunk), receiver))
	});
	Ok(HttpResponse::Ok()
		.content_type("application/zip")
		.insert_header((CONTENT_DISPOSITION, format!("attachment; filename=\"{}.zip\"", info.id)))
		.streaming(body))
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}/backups/{snap}",
	tag = "backups",
	params(
		("id" = i64, Path, description = "Instance id"),
		("snap" = String, Path, description = "Snapshot id"),
	),
	responses(
		(status = 204, description = "Snapshot deleted"),
		(status = 403, description = "Missing backups.delete"),
		(status = 404, description = "Snapshot not found"),
	)
)]
#[delete("/instances/{id}/backups/{snap}")]
pub async fn delete_backup(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
) -> ApiResult<HttpResponse> {
	let (instance, snap) = path.into_inner();
	require_instance_perm(&state, &user, instance, Permission::BACKUPS_DELETE).await?;
	state.backups.delete(instance, &snap).await?;
	Ok(HttpResponse::NoContent().finish())
}

struct ChannelWriter(mpsc::Sender<Bytes>);

impl Write for ChannelWriter {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.0
			.blocking_send(Bytes::copy_from_slice(buf))
			.map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "download closed"))?;
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}
