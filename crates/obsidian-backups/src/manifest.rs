use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, Result};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::locked;
use crate::model::SnapshotInfo;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Manifest {
	pub id: String,
	pub label: String,
	pub created_at: DateTime<Utc>,
	pub file_count: u64,
	pub size_bytes: u64,
	#[serde(default)]
	pub excludes: Vec<String>,
	pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Entry {
	pub path: String,
	pub dir: bool,
	pub size: u64,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub chunks: Vec<String>,
}

impl Manifest {
	pub fn info(&self) -> SnapshotInfo {
		SnapshotInfo {
			id: self.id.clone(),
			label: self.label.clone(),
			file_count: self.file_count,
			size_bytes: self.size_bytes,
			created_at: self.created_at,
		}
	}
}

pub(crate) fn check_id(id: &str) -> Result<()> {
	if !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-') {
		Ok(())
	} else {
		Err(CommonError::Invalid(format!("invalid snapshot id `{id}`")))
	}
}

fn file_path(snapshots: &Path, id: &str) -> PathBuf {
	snapshots.join(format!("{id}.json"))
}

pub(crate) fn read(snapshots: &Path, id: &str) -> Result<Manifest> {
	let file = match File::open(file_path(snapshots, id)) {
		Ok(file) => file,
		Err(error) if error.kind() == ErrorKind::NotFound => {
			return Err(CommonError::NotFound(format!("snapshot {id}")));
		}
		Err(error) => return Err(error.into()),
	};
	Ok(serde_json::from_reader(BufReader::new(file))?)
}

pub(crate) fn write(snapshots: &Path, manifest: &Manifest) -> Result<()> {
	fs::create_dir_all(snapshots)?;
	let mut temp = NamedTempFile::new_in(snapshots)?;
	{
		let mut out = BufWriter::new(temp.as_file_mut());
		serde_json::to_writer(&mut out, manifest)?;
		out.flush()?;
	}
	temp.as_file().sync_all()?;
	locked::persist(temp, &file_path(snapshots, &manifest.id))?;
	Ok(())
}

pub(crate) fn remove(snapshots: &Path, id: &str) -> Result<()> {
	match locked::retry(|| fs::remove_file(file_path(snapshots, id))) {
		Ok(()) => Ok(()),
		Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
		Err(error) => Err(error.into()),
	}
}

pub(crate) fn list_ids(snapshots: &Path) -> Result<Vec<String>> {
	let entries = match fs::read_dir(snapshots) {
		Ok(entries) => entries,
		Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
		Err(error) => return Err(error.into()),
	};
	let mut ids = Vec::new();
	for entry in entries {
		let path = entry?.path();
		if path.extension().is_some_and(|ext| ext == "json")
			&& let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
		{
			ids.push(stem.to_owned());
		}
	}
	Ok(ids)
}

// An unreadable manifest aborts collection so a sweep never deletes chunks it may still reference.
pub(crate) fn referenced_chunks(snapshots: &Path) -> Result<HashSet<String>> {
	let mut hashes = HashSet::new();
	for id in list_ids(snapshots)? {
		let manifest = read(snapshots, &id)?;
		hashes.extend(manifest.entries.into_iter().flat_map(|entry| entry.chunks));
	}
	Ok(hashes)
}
