use std::path::{Path, PathBuf};

use crate::Result;

#[derive(Debug, Clone)]
pub struct Paths {
	pub data_dir: PathBuf,
	pub instances_dir: PathBuf,
	pub java_dir: PathBuf,
	pub cache_dir: PathBuf,
	pub backups_dir: PathBuf,
	pub runners_dir: PathBuf,
	pub uploads_dir: PathBuf,
}

impl Paths {
	pub fn new(data_dir: impl Into<PathBuf>) -> Result<Self> {
		let data_dir = data_dir.into();
		let paths = Self {
			instances_dir: data_dir.join("instances"),
			java_dir: data_dir.join("java"),
			cache_dir: data_dir.join("cache"),
			backups_dir: data_dir.join("backups"),
			runners_dir: data_dir.join("runners"),
			uploads_dir: data_dir.join("uploads"),
			data_dir,
		};
		for dir in [
			&paths.data_dir,
			&paths.instances_dir,
			&paths.java_dir,
			&paths.cache_dir,
			&paths.backups_dir,
			&paths.runners_dir,
			&paths.uploads_dir,
		] {
			std::fs::create_dir_all(dir)?;
		}
		Ok(paths)
	}

	pub fn database_file(&self) -> PathBuf {
		self.data_dir.join("obsidian.db")
	}

	pub fn config_file(&self) -> PathBuf {
		self.data_dir.join("config.toml")
	}

	pub fn instance_dir(&self, id: crate::InstanceId) -> PathBuf {
		self.instances_dir.join(id.to_string())
	}

	pub fn data_dir(&self) -> &Path {
		&self.data_dir
	}
}
