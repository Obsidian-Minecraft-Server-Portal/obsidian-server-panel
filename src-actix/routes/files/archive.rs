use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use actix_files::NamedFile;
use actix_web::http::header::ContentDisposition;
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_sftp::jail;
use serde::{Deserialize, Serialize};
use tokio::fs;
use utoipa::{IntoParams, ToSchema};

use super::browse::PathQuery;
use super::compress::{self, ArchiveJob, Reservation};
use super::stored_zip;
use super::util::{authorize, blocking, invalid, io_err, parent_path, resolve};
use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::routes::streaming;
use crate::state::AppState;

const MAX_SELECTION: usize = 512;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ArchiveRequest {
	/// Entries to include. Folders are added recursively. Each must be inside the folder the zip is written to.
	pub paths: Vec<String>,
	/// Destination `.zip` path. Must not exist yet.
	pub dest: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArchiveStarted {
	pub task_id: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExtractRequest {
	/// Archive to extract (`.zip`, `.tar.gz` or `.tgz`).
	pub path: String,
	/// Destination directory. Defaults to the archive's folder.
	pub dest: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArchiveResult {
	pub path: String,
	pub entries: usize,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SelectionQuery {
	/// Folder that holds the selection. Entries in the zip are named relative to it.
	#[serde(default)]
	pub dir: String,
	/// JSON array of instance-relative paths. Each must be inside `dir`.
	pub paths: String,
}

#[derive(Debug, Clone, Copy)]
enum ArchiveKind {
	Zip,
	TarGz,
}

impl ArchiveKind {
	fn detect(path: &str) -> Result<Self> {
		let lower = path.to_ascii_lowercase();
		if lower.ends_with(".zip") {
			Ok(Self::Zip)
		} else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
			Ok(Self::TarGz)
		} else {
			Err(CommonError::Invalid("unsupported archive format, expected .zip, .tar.gz or .tgz".into()))
		}
	}
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/files/archive",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = ArchiveRequest,
	responses(
		(status = 202, description = "Zip started as a background task", body = ArchiveStarted),
		(status = 400, description = "Invalid entries or archive name"),
		(status = 404, description = "An entry does not exist"),
		(status = 409, description = "The destination exists or is already being written"),
	)
)]
#[post("/archive")]
pub async fn archive_entries(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<ArchiveRequest>,
) -> ApiResult<HttpResponse> {
	let root = authorize(&state, &user, *id, Permission::FILES_READ | Permission::FILES_WRITE).await?;
	let ArchiveRequest { paths, dest: rel } = body.into_inner();
	if paths.is_empty() {
		return Err(CommonError::Invalid("nothing to compress".into()).into());
	}
	ensure_zip_name(&rel)?;
	let dest = resolve(&root, &rel).await?;
	let folder = dest.parent().unwrap_or(&root).to_path_buf();
	let mut sources = Vec::with_capacity(paths.len());
	for path in &paths {
		sources.push(resolve_inside(&root, path, &folder).await?);
	}
	let reservation = Reservation::claim(&dest)?;
	let name = file_name(&dest, "archive.zip");
	let task = state.tasks.create("file_archive", format!("Compress to {name}"), Some(user.id));
	let started = ArchiveStarted { task_id: task.id() };
	compress::spawn(task, ArchiveJob { rel, sources, folder, dest, reservation });
	Ok(HttpResponse::Accepted().json(started))
}

#[utoipa::path(
	post,
	path = "/api/v1/instances/{id}/files/extract",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id")),
	request_body = ExtractRequest,
	responses((status = 200, description = "Archive extracted", body = ArchiveResult), (status = 400, description = "Invalid or unsupported archive"))
)]
#[post("/extract")]
pub async fn extract_archive(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<ExtractRequest>,
) -> ApiResult<web::Json<ArchiveResult>> {
	let root = authorize(&state, &user, *id, Permission::FILES_READ | Permission::FILES_WRITE).await?;
	let kind = ArchiveKind::detect(&body.path)?;
	let archive = resolve(&root, &body.path).await?;
	let dest_rel = body.dest.clone().unwrap_or_else(|| parent_path(&body.path));
	resolve(&root, &dest_rel).await?;
	let entries = blocking(move || match kind {
		ArchiveKind::Zip => extract_zip(&root, &archive, &dest_rel),
		ArchiveKind::TarGz => extract_tar_gz(&root, &archive, &dest_rel),
	})
	.await?;
	Ok(web::Json(ArchiveResult { path: body.path.clone(), entries }))
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/files/download",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id"), PathQuery),
	responses(
		(status = 200, description = "File contents, or a streamed zip of a folder", content_type = "application/octet-stream"),
		(status = 404, description = "Path does not exist"),
	)
)]
#[get("/download")]
pub async fn download(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	query: web::Query<PathQuery>,
	req: HttpRequest,
) -> ApiResult<HttpResponse> {
	let root = authorize(&state, &user, *id, Permission::FILES_READ).await?;
	let path = resolve(&root, &query.path).await?;
	let metadata = fs::metadata(&path).await.map_err(io_err)?;
	if !metadata.is_dir() {
		let name = file_name(&path, "download");
		let file = fs::File::open(&path).await.map_err(io_err)?.into_std().await;
		let named = NamedFile::from_file(file, &path).map_err(io_err)?;
		return Ok(named.set_content_disposition(ContentDisposition::attachment(name)).into_response(&req));
	}

	let name = format!("{}.zip", file_name(&path, "instance"));
	let folder = path.parent().unwrap_or(&root).to_path_buf();
	stream_zip(&name, vec![path], folder).await
}

#[utoipa::path(
	get,
	path = "/api/v1/instances/{id}/files/zip",
	tag = "files",
	params(("id" = i64, Path, description = "Instance id"), SelectionQuery),
	responses(
		(status = 200, description = "Streamed zip of the selection", content_type = "application/zip"),
		(status = 400, description = "Invalid selection"),
		(status = 404, description = "An entry does not exist"),
	)
)]
#[get("/zip")]
pub async fn zip_selection(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	query: web::Query<SelectionQuery>,
) -> ApiResult<HttpResponse> {
	let root = authorize(&state, &user, *id, Permission::FILES_READ).await?;
	let relatives: Vec<String> = serde_json::from_str(&query.paths).map_err(invalid)?;
	if relatives.is_empty() || relatives.len() > MAX_SELECTION {
		return Err(CommonError::Invalid(format!("select between 1 and {MAX_SELECTION} entries")).into());
	}
	let dir = query.dir.trim_matches('/');
	let folder = resolve(&root, dir).await?;
	if !fs::metadata(&folder).await.map_err(io_err)?.is_dir() {
		return Err(CommonError::Invalid("the selection must be inside a folder".into()).into());
	}
	let mut sources = Vec::with_capacity(relatives.len());
	for relative in &relatives {
		sources.push(resolve_inside(&root, relative, &folder).await?);
	}
	stream_zip(&selection_name(dir), sources, folder).await
}

fn file_name(path: &Path, fallback: &str) -> String {
	path.file_name().map_or_else(|| fallback.to_owned(), |name| name.to_string_lossy().into_owned())
}

fn ensure_zip_name(rel: &str) -> Result<()> {
	if rel.to_ascii_lowercase().ends_with(".zip") {
		Ok(())
	} else {
		Err(CommonError::Invalid("the archive name must end in .zip".into()))
	}
}

async fn resolve_inside(root: &Path, relative: &str, folder: &Path) -> ApiResult<PathBuf> {
	let path = resolve(root, relative).await?;
	if !path.starts_with(folder) {
		return Err(CommonError::Invalid(format!("`{relative}` is outside the folder of the archive")).into());
	}
	fs::symlink_metadata(&path).await.map_err(io_err)?;
	Ok(path)
}

fn selection_name(dir: &str) -> String {
	let folder = dir.rsplit('/').next().filter(|part| !part.is_empty()).unwrap_or("files");
	format!("{folder}.zip")
}

/// The tree is walked before responding, so a failing walk is an error response, not a truncated download.
async fn stream_zip(file_name: &str, sources: Vec<PathBuf>, folder: PathBuf) -> ApiResult<HttpResponse> {
	let plan = blocking(move || stored_zip::collect(&sources, &folder, &[])).await?;
	let (writer, body) = streaming::channel();
	tokio::task::spawn_blocking(move || {
		if let Err(error) = stored_zip::write_stream(writer, &plan.entries, |_, _| {}) {
			tracing::warn!(%error, "zip download aborted");
		}
	});
	Ok(streaming::zip_response(file_name, body))
}

fn extract_zip(root: &Path, archive: &Path, dest_rel: &str) -> Result<usize> {
	let mut zip = zip::ZipArchive::new(File::open(archive)?).map_err(invalid)?;
	let mut extracted = 0;
	for index in 0..zip.len() {
		let mut entry = zip.by_index(index).map_err(invalid)?;
		let Some(name) = entry.enclosed_name() else { continue };
		if entry.is_symlink() {
			continue;
		}
		let target = jail::resolve(root, Path::new(dest_rel).join(name))?;
		if entry.is_dir() {
			std::fs::create_dir_all(&target)?;
		} else {
			if let Some(parent) = target.parent() {
				std::fs::create_dir_all(parent)?;
			}
			io::copy(&mut entry, &mut File::create(&target)?)?;
		}
		extracted += 1;
	}
	Ok(extracted)
}

fn extract_tar_gz(root: &Path, archive: &Path, dest_rel: &str) -> Result<usize> {
	let decoder = flate2::read::GzDecoder::new(File::open(archive)?);
	let mut tar = tar::Archive::new(decoder);
	let mut extracted = 0;
	for entry in tar.entries()? {
		let mut entry = entry?;
		let kind = entry.header().entry_type();
		if !(kind.is_file() || kind.is_dir()) {
			continue;
		}
		let name = entry.path()?.into_owned();
		let target = jail::resolve(root, Path::new(dest_rel).join(name))?;
		if let Some(parent) = target.parent() {
			std::fs::create_dir_all(parent)?;
		}
		entry.unpack(&target)?;
		extracted += 1;
	}
	Ok(extracted)
}
