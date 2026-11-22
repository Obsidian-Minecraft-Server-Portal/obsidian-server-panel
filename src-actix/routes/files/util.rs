use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use actix_web::web;
use obsidian_auth::Permission;
use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_sftp::jail;

use crate::auth::{AuthUser, require_instance_perm};
use crate::error::ApiResult;
use crate::state::AppState;

/// Checks `perm` on the instance and returns its root directory.
pub async fn authorize(state: &AppState, user: &AuthUser, id: InstanceId, perm: Permission) -> ApiResult<PathBuf> {
	require_instance_perm(state, user, id, perm).await?;
	Ok(state.paths.instance_dir(id))
}

pub async fn resolve(root: &Path, relative: &str) -> ApiResult<PathBuf> {
	Ok(jail::resolve_async(root, relative).await?)
}

pub fn ensure_not_root(root: &Path, path: &Path) -> ApiResult<()> {
	if path == root {
		return Err(CommonError::Forbidden("the instance root cannot be modified".into()).into());
	}
	Ok(())
}

pub fn io_err(error: std::io::Error) -> CommonError {
	match error.kind() {
		ErrorKind::NotFound => CommonError::NotFound("path does not exist".into()),
		ErrorKind::AlreadyExists => CommonError::Conflict("path already exists".into()),
		ErrorKind::PermissionDenied => CommonError::Forbidden("permission denied by the file system".into()),
		ErrorKind::IsADirectory | ErrorKind::NotADirectory | ErrorKind::DirectoryNotEmpty => {
			CommonError::Invalid(error.to_string())
		}
		_ => CommonError::Io(error),
	}
}

pub fn invalid(error: impl std::fmt::Display) -> CommonError {
	CommonError::Invalid(error.to_string())
}

pub async fn blocking<T, F>(task: F) -> ApiResult<T>
where
	F: FnOnce() -> Result<T> + Send + 'static,
	T: Send + 'static,
{
	let result = web::block(task).await.map_err(|e| CommonError::Other(e.to_string()))?;
	Ok(result?)
}

/// Joins a directory path and an entry name into an instance-relative path using `/`.
pub fn child_path(parent: &str, name: &str) -> String {
	let parent = parent.trim_matches('/');
	if parent.is_empty() { name.to_owned() } else { format!("{parent}/{name}") }
}

pub fn parent_path(path: &str) -> String {
	Path::new(path.trim_matches('/')).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn modified_secs(metadata: &std::fs::Metadata) -> i64 {
	metadata
		.modified()
		.ok()
		.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
		.map_or(0, |duration| duration.as_secs() as i64)
}
