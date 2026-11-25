use std::fs::File;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::Utc;
use fastcdc::v2020::StreamCDC;
use ignore::WalkBuilder;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use obsidian_common::{CommonError, Result, TaskHandle};
use rayon::prelude::*;

use crate::chunks::ChunkStore;
use crate::manifest::{Entry, Manifest};
use crate::model::PROGRESS_STEP;

const MIN_CHUNK: usize = 64 * 1024;
const AVG_CHUNK: usize = 256 * 1024;
const MAX_CHUNK: usize = 1024 * 1024;
/// The rendered world map is derived from the world, so snapshots leave it out.
const ALWAYS_EXCLUDED: [&str; 2] = ["session.lock", "obsidian-map/"];

struct Scanned {
	path: String,
	source: PathBuf,
	dir: bool,
}

pub(crate) fn build(
	source: &Path,
	chunks: &ChunkStore,
	excludes: Vec<String>,
	id: String,
	label: &str,
	task: &TaskHandle,
) -> Result<Manifest> {
	let scanned = scan(source, &excludes)?;
	let total = scanned.iter().filter(|item| !item.dir).count();
	let done = AtomicUsize::new(0);
	let stored = scanned
		.into_par_iter()
		.map(|item| -> Result<Option<Entry>> {
			if item.dir {
				return Ok(Some(Entry { path: item.path, dir: true, size: 0, chunks: Vec::new() }));
			}
			let stored = store_file(&item.source, chunks)?;
			let finished = done.fetch_add(1, Ordering::Relaxed) + 1;
			if finished.is_multiple_of(PROGRESS_STEP) || finished == total {
				task.progress(finished as f32 / total as f32, format!("{finished}/{total} files"));
			}
			Ok(stored.map(|(chunks, size)| Entry { path: item.path, dir: false, size, chunks }))
		})
		.collect::<Result<Vec<Option<Entry>>>>()?;
	let mut entries: Vec<Entry> = stored.into_iter().flatten().collect();
	entries.sort_unstable_by(|a, b| a.path.cmp(&b.path));
	let file_count = entries.iter().filter(|entry| !entry.dir).count() as u64;
	let size_bytes = entries.iter().map(|entry| entry.size).sum();
	Ok(Manifest { id, label: label.to_owned(), created_at: Utc::now(), file_count, size_bytes, excludes, entries })
}

fn scan(source: &Path, excludes: &[String]) -> Result<Vec<Scanned>> {
	let rules = excludes_matcher(source, excludes)?;
	let walker = WalkBuilder::new(source)
		.standard_filters(false)
		.filter_entry(move |entry| {
			let is_dir = entry.file_type().is_some_and(|kind| kind.is_dir());
			!rules.matched_path_or_any_parents(entry.path(), is_dir).is_ignore()
		})
		.build();
	let mut scanned = Vec::new();
	for entry in walker {
		let entry = entry.map_err(|error| CommonError::Other(error.to_string()))?;
		let Some(kind) = entry.file_type() else { continue };
		if entry.depth() == 0 || kind.is_symlink() || !(kind.is_dir() || kind.is_file()) {
			continue;
		}
		let dir = kind.is_dir();
		let relative = entry.path().strip_prefix(source).map_err(|error| CommonError::Other(error.to_string()))?;
		let path = portable(relative);
		scanned.push(Scanned { path, source: entry.into_path(), dir });
	}
	Ok(scanned)
}

pub(crate) fn excludes_matcher(root: &Path, patterns: &[String]) -> Result<Gitignore> {
	let mut rules = GitignoreBuilder::new(root);
	for pattern in ALWAYS_EXCLUDED.into_iter().chain(patterns.iter().map(String::as_str)) {
		rules
			.add_line(None, pattern)
			.map_err(|error| CommonError::Invalid(format!("exclude `{pattern}`: {error}")))?;
	}
	rules.build().map_err(|error| CommonError::Invalid(error.to_string()))
}

fn store_file(path: &Path, chunks: &ChunkStore) -> Result<Option<(Vec<String>, u64)>> {
	let file = match File::open(path) {
		Ok(file) => file,
		Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
		Err(error) => return Err(error.into()),
	};
	let mut hashes = Vec::new();
	let mut size = 0;
	for chunk in StreamCDC::new(file, MIN_CHUNK, AVG_CHUNK, MAX_CHUNK) {
		let chunk = chunk.map_err(|error| CommonError::Other(error.to_string()))?;
		hashes.push(chunks.put(&chunk.data)?);
		size += chunk.length as u64;
	}
	Ok(Some((hashes, size)))
}

fn portable(relative: &Path) -> String {
	relative
		.components()
		.filter_map(|component| match component {
			Component::Normal(part) => Some(part.to_string_lossy()),
			_ => None,
		})
		.collect::<Vec<_>>()
		.join("/")
}
