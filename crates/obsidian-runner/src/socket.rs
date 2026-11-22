use std::path::PathBuf;

use interprocess::local_socket::{GenericFilePath, Name, ToFsName};
use obsidian_common::{InstanceId, Paths, Result};

#[cfg(unix)]
pub fn path(paths: &Paths, id: InstanceId) -> PathBuf {
	paths.runners_dir.join(format!("{id}.sock"))
}

#[cfg(windows)]
pub fn path(_paths: &Paths, id: InstanceId) -> PathBuf {
	PathBuf::from(format!(r"\\.\pipe\obsidian-runner-{id}"))
}

pub fn name(path: PathBuf) -> Result<Name<'static>> {
	Ok(path.to_fs_name::<GenericFilePath>()?)
}
