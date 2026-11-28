//! Writes a file through a temporary sibling, so a reader sees the old bytes or the new ones and never a partial file.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const REPLACE_ATTEMPTS: u32 = 5;
const REPLACE_PAUSE: Duration = Duration::from_millis(40);

/// Replaces the file at `path`. If the file stays locked after a few retries, the bytes are written in place instead, so a
/// reader holding the file never fails a render.
pub fn write(path: &Path, bytes: impl AsRef<[u8]>) -> io::Result<()> {
	let bytes = bytes.as_ref();
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	let mut name = path.as_os_str().to_owned();
	name.push(".tmp");
	let temporary = PathBuf::from(name);
	if let Err(error) = fs::write(&temporary, bytes) {
		let _ = fs::remove_file(&temporary);
		return Err(error);
	}
	match replace(&temporary, path) {
		Ok(()) => Ok(()),
		Err(_) => {
			let _ = fs::remove_file(&temporary);
			fs::write(path, bytes)
		}
	}
}

/// Windows refuses a rename over a file another handle holds unless that handle allows deletion, and a virus scanner can
/// hold a freshly written file for a moment. A short retry covers that.
fn replace(temporary: &Path, path: &Path) -> io::Result<()> {
	let mut attempt = 1_u32;
	loop {
		match fs::rename(temporary, path) {
			Ok(()) => return Ok(()),
			Err(error) if attempt >= REPLACE_ATTEMPTS => return Err(error),
			Err(_) => {
				thread::sleep(REPLACE_PAUSE * attempt);
				attempt += 1;
			}
		}
	}
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

	#[cfg(windows)]
	#[test]
	fn a_file_held_without_delete_sharing_is_still_replaced() {
		use std::os::windows::fs::OpenOptionsExt;
		const SHARE_READ_AND_WRITE: u32 = 0x1 | 0x2;
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("blocks.json");
		write(&path, b"first").unwrap();
		let _reader = std::fs::OpenOptions::new().read(true).share_mode(SHARE_READ_AND_WRITE).open(&path).unwrap();
		write(&path, b"second").unwrap();
		assert_eq!(std::fs::read(&path).unwrap(), b"second");
	}
}
