//! Writes a file through a temporary sibling, so a reader sees the old bytes or the new ones and never a partial file.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn write(path: &Path, bytes: impl AsRef<[u8]>) -> io::Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	let mut name = path.as_os_str().to_owned();
	name.push(".tmp");
	let temporary = PathBuf::from(name);
	let result = fs::write(&temporary, bytes).and_then(|()| fs::rename(&temporary, path));
	if result.is_err() {
		let _ = fs::remove_file(&temporary);
	}
	result
}

#[cfg(test)]
mod tests {
	use super::write;

	#[test]
	fn a_rewrite_replaces_the_file_and_leaves_no_temporary_behind() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("nested").join("states.json");
		write(&path, b"first").unwrap();
		write(&path, b"second").unwrap();
		assert_eq!(std::fs::read(&path).unwrap(), b"second");
		assert!(!dir.path().join("nested").join("states.json.tmp").exists());
	}
}
