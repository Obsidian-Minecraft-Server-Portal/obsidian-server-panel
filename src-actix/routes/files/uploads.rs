use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::{Duration, SystemTime};

use actix_web::{HttpResponse, delete, get, post, put, web};
use futures_util::StreamExt;
use obsidian_auth::{Permission, generate_token};
use obsidian_common::{CommonError, InstanceId};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use utoipa::ToSchema;

use super::util::{authorize, ensure_not_root, io_err, resolve};
use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

const UPLOAD_ID_LEN: usize = 64;
const UPLOAD_TTL: Duration = Duration::from_secs(24 * 60 * 60);

static ACTIVE: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Mutex::default);

/// Serializes writers of one upload session so concurrent chunks cannot interleave.
struct UploadLock(String);

impl UploadLock {
	fn acquire(upload_id: &str) -> ApiResult<Self> {
		let mut active = ACTIVE.lock().unwrap_or_else(PoisonError::into_inner);
		if !active.insert(upload_id.to_owned()) {
			return Err(CommonError::Conflict("another request is writing this upload".into()).into());
		}
		Ok(Self(upload_id.to_owned()))
	}

	fn is_active(upload_id: &str) -> bool {
		ACTIVE.lock().unwrap_or_else(PoisonError::into_inner).contains(upload_id)
	}
}

impl Drop for UploadLock {
	fn drop(&mut self) {
		ACTIVE.lock().unwrap_or_else(PoisonError::into_inner).remove(&self.0);
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadMeta {
	instance: InstanceId,
	path: String,
	size: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUploadRequest {
	/// Destination file path relative to the instance root. Missing parent folders are created when the upload finishes.
	pub path: String,
	pub size: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OffsetQuery {
	pub offset: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadStatus {
	pub upload_id: String,
	pub path: String,
	pub size: u64,
	/// Bytes already received. Resume uploading from this offset.
	pub offset: u64,
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/files/uploads",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = CreateUploadRequest,
	responses((status = 201, description = "Upload session created", body = UploadStatus))
)]
#[post("/uploads")]
pub async fn create_upload(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<CreateUploadRequest>,
) -> ApiResult<HttpResponse> {
	let root = authorize(&state, &user, *id, Permission::FILES_WRITE).await?;
	let target = resolve(&root, &body.path).await?;
	ensure_not_root(&root, &target)?;
	tokio::spawn(sweep_stale(state.paths.uploads_dir.clone()));
	let upload_id = generate_token();
	let meta = UploadMeta { instance: *id, path: body.path.clone(), size: body.size };
	fs::write(part_path(&state, &upload_id), b"").await.map_err(io_err)?;
	fs::write(meta_path(&state, &upload_id), serde_json::to_vec(&meta).map_err(CommonError::from)?).await.map_err(io_err)?;
	Ok(HttpResponse::Created().json(status(upload_id, meta, 0)))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/files/uploads/{upload_id}",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id"), ("upload_id" = String, Path, description = "Upload session id")),
	responses((status = 200, description = "Upload progress", body = UploadStatus), (status = 404, description = "Unknown upload"))
)]
#[get("/uploads/{upload_id}")]
pub async fn upload_status(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
) -> ApiResult<web::Json<UploadStatus>> {
	let (id, upload_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_WRITE).await?;
	let meta = load_meta(&state, id, &upload_id).await?;
	let offset = part_len(&state, &upload_id).await?;
	Ok(web::Json(status(upload_id, meta, offset)))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/files/uploads/{upload_id}",
	tag = "files",
	params(
		("id" = i64, Path, description = "Instance id"),
		("upload_id" = String, Path, description = "Upload session id"),
		("offset" = u64, Query, description = "Byte offset this chunk starts at; must equal the current offset"),
	),
	request_body(content = Vec<u8>, content_type = "application/octet-stream"),
	responses((status = 200, description = "Chunk stored", body = UploadStatus), (status = 409, description = "Offset mismatch or concurrent write"))
)]
#[put("/uploads/{upload_id}")]
pub async fn upload_chunk(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
	query: web::Query<OffsetQuery>,
	mut payload: web::Payload,
) -> ApiResult<web::Json<UploadStatus>> {
	let (id, upload_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_WRITE).await?;
	let meta = load_meta(&state, id, &upload_id).await?;
	let _lock = UploadLock::acquire(&upload_id)?;
	let received = part_len(&state, &upload_id).await?;
	if query.offset != received {
		return Err(CommonError::Conflict(format!("upload offset mismatch, expected {received}")).into());
	}
	let mut file = OpenOptions::new().append(true).open(part_path(&state, &upload_id)).await.map_err(io_err)?;
	let mut offset = received;
	while let Some(chunk) = payload.next().await {
		let chunk = chunk.map_err(|e| CommonError::Invalid(e.to_string()))?;
		offset += chunk.len() as u64;
		if offset > meta.size {
			return Err(CommonError::Invalid("chunk exceeds the declared upload size".into()).into());
		}
		file.write_all(&chunk).await.map_err(io_err)?;
	}
	file.flush().await.map_err(io_err)?;
	Ok(web::Json(status(upload_id, meta, offset)))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/files/uploads/{upload_id}/finalize",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id"), ("upload_id" = String, Path, description = "Upload session id")),
	responses((status = 200, description = "File moved into place", body = UploadStatus), (status = 409, description = "Upload incomplete"))
)]
#[post("/uploads/{upload_id}/finalize")]
pub async fn finalize_upload(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
) -> ApiResult<web::Json<UploadStatus>> {
	let (id, upload_id) = path.into_inner();
	let root = authorize(&state, &user, id, Permission::FILES_WRITE).await?;
	let meta = load_meta(&state, id, &upload_id).await?;
	let _lock = UploadLock::acquire(&upload_id)?;
	let received = part_len(&state, &upload_id).await?;
	if received != meta.size {
		return Err(CommonError::Conflict(format!("upload incomplete: {received} of {} bytes", meta.size)).into());
	}
	let target = resolve(&root, &meta.path).await?;
	ensure_not_root(&root, &target)?;
	if let Some(parent) = target.parent() {
		fs::create_dir_all(parent).await.map_err(io_err)?;
	}
	let part = part_path(&state, &upload_id);
	if fs::rename(&part, &target).await.is_err() {
		fs::copy(&part, &target).await.map_err(io_err)?;
		let _ = fs::remove_file(&part).await;
	}
	let _ = fs::remove_file(meta_path(&state, &upload_id)).await;
	Ok(web::Json(status(upload_id, meta, received)))
}

#[utoipa::path(
	delete,
	path = "/api/v1/instances/{id}/files/uploads/{upload_id}",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id"), ("upload_id" = String, Path, description = "Upload session id")),
	responses((status = 204, description = "Upload cancelled"))
)]
#[delete("/uploads/{upload_id}")]
pub async fn cancel_upload(
	state: web::Data<AppState>,
	user: AuthUser,
	path: web::Path<(InstanceId, String)>,
) -> ApiResult<HttpResponse> {
	let (id, upload_id) = path.into_inner();
	require_instance_perm(&state, &user, id, Permission::FILES_WRITE).await?;
	load_meta(&state, id, &upload_id).await?;
	let _lock = UploadLock::acquire(&upload_id)?;
	let _ = fs::remove_file(part_path(&state, &upload_id)).await;
	let _ = fs::remove_file(meta_path(&state, &upload_id)).await;
	Ok(HttpResponse::NoContent().finish())
}

fn status(upload_id: String, meta: UploadMeta, offset: u64) -> UploadStatus {
	UploadStatus { upload_id, path: meta.path, size: meta.size, offset }
}

fn valid_upload_id(upload_id: &str) -> bool {
	upload_id.len() == UPLOAD_ID_LEN && upload_id.bytes().all(|b| b.is_ascii_hexdigit())
}

fn part_path(state: &AppState, upload_id: &str) -> PathBuf {
	state.paths.uploads_dir.join(format!("{upload_id}.part"))
}

fn meta_path(state: &AppState, upload_id: &str) -> PathBuf {
	state.paths.uploads_dir.join(format!("{upload_id}.json"))
}

async fn part_len(state: &AppState, upload_id: &str) -> ApiResult<u64> {
	Ok(fs::metadata(part_path(state, upload_id)).await.map_err(io_err)?.len())
}

async fn load_meta(state: &AppState, id: InstanceId, upload_id: &str) -> ApiResult<UploadMeta> {
	let missing = || CommonError::NotFound("upload does not exist".into());
	if !valid_upload_id(upload_id) {
		return Err(missing().into());
	}
	let bytes = fs::read(meta_path(state, upload_id)).await.map_err(|_| missing())?;
	let meta: UploadMeta = serde_json::from_slice(&bytes).map_err(CommonError::from)?;
	if meta.instance != id {
		return Err(missing().into());
	}
	Ok(meta)
}

async fn sweep_stale(dir: PathBuf) {
	let Ok(mut entries) = fs::read_dir(&dir).await else { return };
	let cutoff = SystemTime::now() - UPLOAD_TTL;
	while let Ok(Some(entry)) = entries.next_entry().await {
		let path = entry.path();
		let Some(upload_id) = session_id(&path) else { continue };
		let part = dir.join(format!("{upload_id}.part"));
		let stale = older_than(&part, cutoff).await.unwrap_or(true) && older_than(&path, cutoff).await.unwrap_or(false);
		if stale && !UploadLock::is_active(upload_id) {
			let _ = fs::remove_file(&path).await;
		}
	}
}

async fn older_than(path: &Path, cutoff: SystemTime) -> Option<bool> {
	let modified = fs::metadata(path).await.ok()?.modified().ok()?;
	Some(modified < cutoff)
}

fn session_id(path: &Path) -> Option<&str> {
	let extension = path.extension()?.to_str()?;
	let stem = path.file_stem()?.to_str()?;
	(matches!(extension, "part" | "json") && valid_upload_id(stem)).then_some(stem)
}
