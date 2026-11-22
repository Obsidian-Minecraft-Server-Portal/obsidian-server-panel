use std::path::{Component, Path, PathBuf};

use obsidian_common::{CommonError, Result};

const ESCAPES: &str = "path escapes the instance directory";

/// Maps a client-supplied relative path onto `root`. `..`, absolute and prefix components are
/// rejected, and existing ancestors are canonicalized so symlinks cannot point outside `root`.
pub fn resolve(root: &Path, relative: impl AsRef<Path>) -> Result<PathBuf> {
	let mut path = root.to_path_buf();
	for component in relative.as_ref().components() {
		match component {
			Component::Normal(part) => path.push(part),
			Component::CurDir | Component::RootDir => {}
			Component::ParentDir | Component::Prefix(_) => return Err(CommonError::Forbidden(ESCAPES.into())),
		}
	}
	ensure_contained(root, &path)?;
	Ok(path)
}

/// [`resolve`] on the blocking thread pool, for use from async code.
pub async fn resolve_async(root: &Path, relative: impl AsRef<Path>) -> Result<PathBuf> {
	let (root, relative) = (root.to_path_buf(), relative.as_ref().to_path_buf());
	tokio::task::spawn_blocking(move || resolve(&root, relative))
		.await
		.map_err(|e| CommonError::Other(e.to_string()))?
}

fn ensure_contained(root: &Path, path: &Path) -> Result<()> {
	let canonical_root = root
		.canonicalize()
		.map_err(|_| CommonError::NotFound("instance directory".into()))?;
	let mut existing = path;
	while existing.symlink_metadata().is_err() {
		existing = existing.parent().ok_or_else(|| CommonError::Forbidden(ESCAPES.into()))?;
	}
	let real = existing
		.canonicalize()
		.map_err(|_| CommonError::Forbidden("path cannot be resolved inside the instance directory".into()))?;
	if real.starts_with(&canonical_root) {
		Ok(())
	} else {
		Err(CommonError::Forbidden(ESCAPES.into()))
	}
}
