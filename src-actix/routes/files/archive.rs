use std::fs::File;
use std::io::{self, BufWriter, Seek, Write};
use std::path::{Component, Path, PathBuf};

use actix_files::NamedFile;
use actix_web::http::header::ContentDisposition;
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_sftp::jail;
use serde::{Deserialize, Serialize};
use tokio::fs;
use utoipa::ToSchema;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use super::browse::PathQuery;
use super::util::{authorize, blocking, invalid, io_err, parent_path, resolve};
use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ArchiveRequest {
	/// Entries to include. Folders are added recursively.
	pub paths: Vec<String>,
	/// Destination `.zip` path. Must not exist yet.
	pub dest: String,
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
	responses((status = 200, description = "Zip created", body = ArchiveResult), (status = 409, description = "Destination exists"))
)]
#[post("/archive")]
pub async fn archive_entries(
	state: web::Data<AppState>,
	user: AuthUser,
	id: web::Path<InstanceId>,
	body: web::Json<ArchiveRequest>,
) -> ApiResult<web::Json<ArchiveResult>> {
	let root = authorize(&state, &user, *id, Permission::FILES_READ | Permission::FILES_WRITE).await?;
	let dest = resolve(&root, &body.dest).await?;
	let mut sources = Vec::with_capacity(body.paths.len());
	for path in &body.paths {
		sources.push(resolve(&root, path).await?);
	}
	let entries = blocking(move || write_zip(&sources, &root, &dest)).await?;
	Ok(web::Json(ArchiveResult { path: body.dest.clone(), entries }))
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
		(status = 200, description = "File contents, or a zip of a folder", content_type = "application/octet-stream"),
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
	let scratch = state.paths.uploads_dir.clone();
	let file = blocking(move || {
		let base = path.parent().unwrap_or(&root);
		let (buffer, _) = zip_into(BufWriter::new(tempfile::tempfile_in(scratch)?), std::slice::from_ref(&path), base, None)?;
		let mut file = buffer.into_inner().map_err(|e| e.into_error())?;
		file.rewind()?;
		Ok(file)
	})
	.await?;
	let named = NamedFile::from_file(file, PathBuf::from(&name)).map_err(io_err)?;
	Ok(named.set_content_disposition(ContentDisposition::attachment(name)).into_response(&req))
}

fn file_name(path: &Path, fallback: &str) -> String {
	path.file_name().map_or_else(|| fallback.to_owned(), |name| name.to_string_lossy().into_owned())
}

fn write_zip(sources: &[PathBuf], base: &Path, dest: &Path) -> Result<usize> {
	let file = std::fs::OpenOptions::new().write(true).create_new(true).open(dest)?;
	let outcome = zip_into(BufWriter::new(file), sources, base, Some(dest))
		.and_then(|(mut buffer, entries)| Ok(buffer.flush().map(|_| entries)?));
	if outcome.is_err() {
		let _ = std::fs::remove_file(dest);
	}
	outcome
}

/// Entry names are relative to `base` with `/` separators. Symlinks are skipped so an archive
/// cannot carry links out of the instance.
fn zip_into<W: Write + Seek>(out: W, sources: &[PathBuf], base: &Path, skip: Option<&Path>) -> Result<(W, usize)> {
	let mut writer = zip::ZipWriter::new(out);
	let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
	let mut entries = 0;
	for source in sources {
		for entry in WalkDir::new(source).follow_links(false) {
			let entry = entry.map_err(|e| CommonError::Other(e.to_string()))?;
			let path = entry.path();
			if Some(path) == skip || entry.file_type().is_symlink() {
				continue;
			}
			let Ok(relative) = path.strip_prefix(base) else { continue };
			let name = zip_name(relative);
			if name.is_empty() {
				continue;
			}
			if entry.file_type().is_dir() {
				writer.add_directory(format!("{name}/"), options).map_err(zip_err)?;
			} else {
				writer.start_file(name, options).map_err(zip_err)?;
				io::copy(&mut File::open(path)?, &mut writer)?;
			}
			entries += 1;
		}
	}
	Ok((writer.finish().map_err(zip_err)?, entries))
}

fn zip_name(relative: &Path) -> String {
	relative
		.components()
		.filter_map(|component| match component {
			Component::Normal(part) => Some(part.to_string_lossy()),
			_ => None,
		})
		.collect::<Vec<_>>()
		.join("/")
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

fn zip_err(error: zip::result::ZipError) -> CommonError {
	match error {
		zip::result::ZipError::Io(io) => CommonError::Io(io),
		other => CommonError::Other(other.to_string()),
	}
}
