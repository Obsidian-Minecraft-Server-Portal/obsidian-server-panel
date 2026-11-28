use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use obsidian_common::{CommonError, Result};
use tempfile::NamedTempFile;

use crate::locked;
use walkdir::WalkDir;

const ZSTD_LEVEL: i32 = 3;

#[derive(Debug, Clone)]
pub(crate) struct ChunkStore {
	dir: PathBuf,
}

impl ChunkStore {
	pub fn new(dir: PathBuf) -> Self {
		Self { dir }
	}

	pub fn put(&self, data: &[u8]) -> io::Result<String> {
		let hash = blake3::hash(data).to_hex().to_string();
		let path = self.path(&hash);
		if path.exists() {
			return Ok(hash);
		}
		let parent = path.parent().ok_or_else(|| io::Error::other("chunk path has no parent"))?;
		fs::create_dir_all(parent)?;
		let packed = zstd::stream::encode_all(data, ZSTD_LEVEL)?;
		let mut temp = NamedTempFile::new_in(parent)?;
		temp.write_all(&packed)?;
		temp.as_file().sync_data()?;
		locked::persist(temp, &path)?;
		Ok(hash)
	}

	pub fn get(&self, hash: &str) -> Result<Vec<u8>> {
		let packed = fs::read(self.path(hash))?;
		let data = zstd::stream::decode_all(packed.as_slice())?;
		if blake3::hash(&data).to_hex().as_str() != hash {
			return Err(CommonError::Other(format!("backup chunk {hash} failed verification")));
		}
		Ok(data)
	}

	pub fn sweep(&self, referenced: &HashSet<String>) -> io::Result<(u64, u64)> {
		let (mut removed, mut freed) = (0, 0);
		for entry in WalkDir::new(&self.dir).min_depth(2).into_iter().filter_map(|entry| entry.ok()) {
			if !entry.file_type().is_file() {
				continue;
			}
			let name = entry.file_name().to_string_lossy();
			if referenced.contains(name.as_ref()) {
				continue;
			}
			freed += entry.metadata().map_err(io::Error::other)?.len();
			locked::retry(|| fs::remove_file(entry.path()))?;
			removed += 1;
		}
		Ok((removed, freed))
	}

	pub fn size(&self) -> u64 {
		dir_size(&self.dir)
	}

	fn path(&self, hash: &str) -> PathBuf {
		self.dir.join(&hash[..2]).join(hash)
	}
}

pub(crate) fn dir_size(dir: &Path) -> u64 {
	WalkDir::new(dir)
		.into_iter()
		.filter_map(|entry| entry.ok())
		.filter(|entry| entry.file_type().is_file())
		.filter_map(|entry| entry.metadata().ok())
		.map(|metadata| metadata.len())
		.sum()
}
