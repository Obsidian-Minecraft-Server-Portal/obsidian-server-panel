use actix_web::{HttpResponse, get, post, put, web};
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId};
use serde::{Deserialize, Serialize};
use tokio::fs;
use utoipa::{IntoParams, ToSchema};

use super::util::{authorize, ensure_not_root, child_path, io_err, modified_secs, resolve};
use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

const MAX_EDIT_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PathQuery {
	/// Path relative to the instance root. Empty or `/` is the root.
	#[serde(default)]
	pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FileEntry {
	pub name: String,
	/// Path relative to the instance root.
	pub path: String,
	pub is_dir: bool,
	pub is_symlink: bool,
	pub size: u64,
	/// Last modification time, unix seconds.
	pub modified: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TextContent {
	pub path: String,
	pub content: String,
	pub size: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WriteTextRequest {
	pub path: String,
	pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PathRequest {
	pub path: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RenameRequest {
	pub from: String,
	pub to: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteRequest {
	pub paths: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteResult {
	pub deleted: usize,
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/files",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id"), PathQuery),
	responses(
		(status = 200, description = "Directory entries, folders first", body = Vec<FileEntry>),
		(status = 403, description = "Missing files.read"),
	)
)]
#[get("")]
pub async fn list_dir(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	query: web::Query<PathQuery>,
) -> ApiResult<web::Json<Vec<FileEntry>>> {
	let root = authorize(&state, &user, *id, Permission::FILES_READ).await?;
	let dir = resolve(&root, &query.path).await?;
	let mut reader = fs::read_dir(&dir).await.map_err(io_err)?;
	let mut entries = Vec::new();
	while let Some(entry) = reader.next_entry().await.map_err(io_err)? {
		let name = entry.file_name().to_string_lossy().into_owned();
		let metadata = entry.metadata().await.map_err(io_err)?;
		entries.push(FileEntry {
			path: child_path(&query.path, &name),
			is_dir: metadata.is_dir(),
			is_symlink: metadata.is_symlink(),
			size: metadata.len(),
			modified: modified_secs(&metadata),
			name,
		});
	}
	entries.sort_by_cached_key(|entry| (!entry.is_dir, entry.name.to_lowercase()));
	Ok(web::Json(entries))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/files/content",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id"), PathQuery),
	responses(
		(status = 200, description = "Text file contents", body = TextContent),
		(status = 400, description = "Directory, too large or not UTF-8"),
	)
)]
#[get("/content")]
pub async fn read_text(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	query: web::Query<PathQuery>,
) -> ApiResult<web::Json<TextContent>> {
	let root = authorize(&state, &user, *id, Permission::FILES_READ).await?;
	let path = resolve(&root, &query.path).await?;
	let metadata = fs::metadata(&path).await.map_err(io_err)?;
	if metadata.is_dir() {
		return Err(CommonError::Invalid("path is a directory".into()).into());
	}
	if metadata.len() > MAX_EDIT_BYTES {
		return Err(CommonError::Invalid("file is larger than 2 MiB and cannot be opened as text".into()).into());
	}
	let bytes = fs::read(&path).await.map_err(io_err)?;
	let content = String::from_utf8(bytes)
		.map_err(|_| CommonError::Invalid("file is not valid UTF-8 text".into()))?;
	Ok(web::Json(TextContent { path: query.path.clone(), content, size: metadata.len() }))
}

#[utoipa::path(
	put,
	path = "/api/v1/instances/{id}/files/content",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = WriteTextRequest,
	responses((status = 204, description = "File written"), (status = 403, description = "Missing files.write"))
)]
#[put("/content")]
pub async fn write_text(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<WriteTextRequest>,
) -> ApiResult<HttpResponse> {
	let root = authorize(&state, &user, *id, Permission::FILES_WRITE).await?;
	let path = resolve(&root, &body.path).await?;
	ensure_not_root(&root, &path)?;
	fs::write(&path, body.content.as_bytes()).await.map_err(io_err)?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/files/mkdir",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = PathRequest,
	responses((status = 204, description = "Directory created"), (status = 409, description = "Already exists"))
)]
#[post("/mkdir")]
pub async fn make_dir(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<PathRequest>,
) -> ApiResult<HttpResponse> {
	let root = authorize(&state, &user, *id, Permission::FILES_WRITE).await?;
	let path = resolve(&root, &body.path).await?;
	ensure_not_root(&root, &path)?;
	fs::create_dir(&path).await.map_err(io_err)?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/files/rename",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = RenameRequest,
	responses((status = 204, description = "Entry moved or renamed"), (status = 409, description = "Target exists"))
)]
#[post("/rename")]
pub async fn rename_entry(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<RenameRequest>,
) -> ApiResult<HttpResponse> {
	let root = authorize(&state, &user, *id, Permission::FILES_WRITE).await?;
	let from = resolve(&root, &body.from).await?;
	let to = resolve(&root, &body.to).await?;
	ensure_not_root(&root, &from)?;
	ensure_not_root(&root, &to)?;
	if fs::try_exists(&to).await.map_err(io_err)? {
		return Err(CommonError::Conflict("target already exists".into()).into());
	}
	fs::rename(&from, &to).await.map_err(io_err)?;
	Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/files/delete",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = DeleteRequest,
	responses((status = 200, description = "Entries removed", body = DeleteResult), (status = 403, description = "Missing files.delete"))
)]
#[post("/delete")]
pub async fn delete_entries(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<DeleteRequest>,
) -> ApiResult<web::Json<DeleteResult>> {
	let root = authorize(&state, &user, *id, Permission::FILES_DELETE).await?;
	let mut targets = Vec::with_capacity(body.paths.len());
	for relative in &body.paths {
		let path = resolve(&root, relative).await?;
		ensure_not_root(&root, &path)?;
		let is_dir = fs::symlink_metadata(&path).await.map_err(io_err)?.is_dir();
		targets.push((path, is_dir));
	}
	for (path, is_dir) in &targets {
		if *is_dir {
			fs::remove_dir_all(path).await.map_err(io_err)?;
		} else {
			fs::remove_file(path).await.map_err(io_err)?;
		}
	}
	Ok(web::Json(DeleteResult { deleted: targets.len() }))
}
