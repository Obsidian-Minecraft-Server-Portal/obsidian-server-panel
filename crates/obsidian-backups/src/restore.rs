use std::collections::HashSet;
use std::fs;
use std::io::{BufWriter, ErrorKind, Write};
use std::path::{Component, Path, PathBuf};

use ignore::WalkBuilder;
use obsidian_common::{CommonError, Result, TaskHandle};
use tempfile::NamedTempFile;
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::chunks::ChunkStore;
use crate::locked;
use crate::manifest::{Entry, Manifest};
use crate::model::{PROGRESS_STEP, TreeEntry};
use crate::snapshot::excludes_matcher;

const EXPORT_BUFFER: usize = 256 * 1024;

pub(crate) fn children(manifest: &Manifest, dir: &str) -> Vec<TreeEntry> {
	let dir = dir.trim_matches('/');
	manifest
		.entries
		.iter()
		.filter(|entry| parent(&entry.path) == dir)
		.map(|entry| TreeEntry {
			path: entry.path.clone(),
			name: entry.path.rsplit_once('/').map_or(entry.path.as_str(), |(_, name)| name).to_owned(),
			is_dir: entry.dir,
			size_bytes: if entry.dir { dir_size(manifest, &entry.path) } else { entry.size },
		})
		.collect()
}

pub(crate) fn restore(
	manifest: &Manifest,
	chunks: &ChunkStore,
	target: &Path,
	subpath: Option<&str>,
	task: &TaskHandle,
) -> Result<()> {
	let subpath = subpath.map(|path| path.trim_matches('/')).filter(|path| !path.is_empty());
	let selected: Vec<&Entry> = manifest.entries.iter().filter(|entry| within(&entry.path, subpath)).collect();
	if subpath.is_some() && selected.is_empty() {
		return Err(CommonError::NotFound(format!("`{}` is not in this snapshot", subpath.unwrap_or_default())));
	}
	fs::create_dir_all(target)?;
	if let Some(sub) = subpath {
		real_ancestors(target, sub)?;
	}
	let total = selected.iter().filter(|entry| !entry.dir).count();
	let mut written: usize = 0;
	for entry in &selected {
		let dest = safe_join(target, &entry.path)?;
		if entry.dir {
			real_dir(&dest)?;
			continue;
		}
		write_file(entry, chunks, &dest)?;
		written += 1;
		if written.is_multiple_of(PROGRESS_STEP) || written == total {
			task.progress(written as f32 / total as f32, entry.path.as_str());
		}
	}
	remove_extras(manifest, target, subpath)
}

pub(crate) fn export_zip<W: Write>(manifest: &Manifest, chunks: &ChunkStore, writer: W) -> Result<()> {
	let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
	let mut zip = ZipWriter::new_stream(BufWriter::with_capacity(EXPORT_BUFFER, writer));
	for entry in &manifest.entries {
		if entry.dir {
			zip.add_directory(&entry.path, options).map_err(zip_error)?;
			continue;
		}
		zip.start_file(&entry.path, options).map_err(zip_error)?;
		for hash in &entry.chunks {
			zip.write_all(&chunks.get(hash)?)?;
		}
	}
	zip.finish().map_err(zip_error)?.into_inner().flush()?;
	Ok(())
}

fn write_file(entry: &Entry, chunks: &ChunkStore, dest: &Path) -> Result<()> {
	let parent = dest.parent().ok_or_else(|| CommonError::Invalid(format!("unsafe snapshot path `{}`", entry.path)))?;
	let mut temp = NamedTempFile::new_in(parent)?;
	{
		let mut out = BufWriter::new(temp.as_file_mut());
		for hash in &entry.chunks {
			out.write_all(&chunks.get(hash)?)?;
		}
		out.flush()?;
	}
	if fs::symlink_metadata(dest).is_ok_and(|meta| meta.is_dir()) {
		locked::retry(|| fs::remove_dir_all(dest))?;
	}
	locked::persist(temp, dest)?;
	Ok(())
}

fn real_dir(path: &Path) -> Result<()> {
	match fs::symlink_metadata(path) {
		Ok(meta) if meta.is_dir() => return Ok(()),
		Ok(_) => locked::retry(|| fs::remove_file(path))?,
		Err(error) if error.kind() == ErrorKind::NotFound => {}
		Err(error) => return Err(error.into()),
	}
	fs::create_dir(path)?;
	Ok(())
}

fn real_ancestors(target: &Path, subpath: &str) -> Result<()> {
	let mut current = target.to_path_buf();
	let mut parts = subpath.split('/').peekable();
	while let Some(part) = parts.next() {
		if parts.peek().is_none() {
			break;
		}
		current = safe_join(&current, part)?;
		real_dir(&current)?;
	}
	Ok(())
}

fn remove_extras(manifest: &Manifest, target: &Path, subpath: Option<&str>) -> Result<()> {
	let scope = match subpath {
		Some(sub) => safe_join(target, sub)?,
		None => target.to_path_buf(),
	};
	if !fs::symlink_metadata(&scope).is_ok_and(|meta| meta.is_dir()) {
		return Ok(());
	}
	let known: HashSet<&str> = manifest.entries.iter().map(|entry| entry.path.as_str()).collect();
	let rules = excludes_matcher(target, &manifest.excludes)?;
	let walker = WalkBuilder::new(&scope)
		.standard_filters(false)
		.filter_entry(move |entry| {
			let is_dir = entry.file_type().is_some_and(|kind| kind.is_dir());
			!rules.matched_path_or_any_parents(entry.path(), is_dir).is_ignore()
		})
		.build();
	let mut stale_dirs = Vec::new();
	for entry in walker {
		let entry = entry.map_err(|error| CommonError::Other(error.to_string()))?;
		let Ok(relative) = entry.path().strip_prefix(target) else { continue };
		let relative = portable(relative);
		if (entry.depth() == 0 && subpath.is_none()) || known.contains(relative.as_str()) {
			continue;
		}
		if entry.file_type().is_some_and(|kind| kind.is_dir()) {
			stale_dirs.push(entry.into_path());
		} else {
			locked::retry(|| fs::remove_file(entry.path()))?;
		}
	}
	for dir in stale_dirs.iter().rev() {
		if let Err(error) = fs::remove_dir(dir)
			&& error.kind() != ErrorKind::DirectoryNotEmpty
		{
			tracing::warn!(path = %dir.display(), %error, "could not remove directory missing from snapshot");
		}
	}
	Ok(())
}

fn zip_error(error: zip::result::ZipError) -> CommonError {
	CommonError::Other(error.to_string())
}

fn parent(path: &str) -> &str {
	path.rsplit_once('/').map_or("", |(parent, _)| parent)
}

fn portable(relative: &Path) -> String {
	relative.components().map(|part| part.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
}

fn dir_size(manifest: &Manifest, dir: &str) -> u64 {
	let prefix = format!("{dir}/");
	manifest
		.entries
		.iter()
		.filter(|entry| !entry.dir && entry.path.starts_with(&prefix))
		.map(|entry| entry.size)
		.sum()
}

fn within(path: &str, subpath: Option<&str>) -> bool {
	match subpath {
		None => true,
		Some(sub) => path == sub || path.strip_prefix(sub).is_some_and(|rest| rest.starts_with('/')),
	}
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
	let path = Path::new(relative);
	if path.components().all(|component| matches!(component, Component::Normal(_))) {
		Ok(root.join(path))
	} else {
		Err(CommonError::Invalid(format!("unsafe snapshot path `{relative}`")))
	}
}
