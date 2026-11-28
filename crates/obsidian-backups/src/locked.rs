//! Removes and replaces files that Windows may refuse for a moment. A virus scanner holds a file it has just written, and the
//! delete or rename fails with access denied until the scan ends, so those operations are retried briefly.

use std::io::{self, ErrorKind};
use std::path::Path;
use std::thread;
use std::time::Duration;

use tempfile::NamedTempFile;

const ATTEMPTS: u32 = 8;
const PAUSE: Duration = Duration::from_millis(25);

fn is_held(error: &io::Error) -> bool {
	error.kind() == ErrorKind::PermissionDenied
}

pub fn retry<T>(mut operation: impl FnMut() -> io::Result<T>) -> io::Result<T> {
	let mut attempt = 1;
	loop {
		match operation() {
			Err(error) if is_held(&error) && attempt < ATTEMPTS => {
				thread::sleep(PAUSE * attempt);
				attempt += 1;
			}
			result => return result,
		}
	}
}

pub fn persist(mut temp: NamedTempFile, dest: &Path) -> io::Result<()> {
	let mut attempt = 1;
	loop {
		match temp.persist(dest) {
			Ok(_) => return Ok(()),
			Err(error) if is_held(&error.error) && attempt < ATTEMPTS => {
				temp = error.file;
				thread::sleep(PAUSE * attempt);
				attempt += 1;
			}
			Err(error) => return Err(error.error),
		}
	}
}

/// Moves `temp` to `dest` only if nothing is there yet. A file already at `dest` is left as it is, and the temporary file is
/// dropped.
pub fn persist_new(mut temp: NamedTempFile, dest: &Path) -> io::Result<()> {
	let mut attempt = 1;
	loop {
		match temp.persist_noclobber(dest) {
			Ok(_) => return Ok(()),
			Err(error) if error.error.kind() == ErrorKind::AlreadyExists => return Ok(()),
			Err(error) if is_held(&error.error) && attempt < ATTEMPTS => {
				temp = error.file;
				thread::sleep(PAUSE * attempt);
				attempt += 1;
			}
			Err(error) => return Err(error.error),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;

	use super::*;

	#[test]
	fn an_existing_file_is_kept_when_new_content_arrives_for_the_same_name() {
		let dir = tempfile::tempdir().unwrap();
		let dest = dir.path().join("chunk");
		std::fs::write(&dest, b"first").unwrap();
		let mut temp = NamedTempFile::new_in(dir.path()).unwrap();
		std::io::Write::write_all(&mut temp, b"second").unwrap();
		persist_new(temp, &dest).unwrap();
		assert_eq!(std::fs::read(&dest).unwrap(), b"first");
	}

	#[test]
	fn a_held_file_is_retried_until_it_is_free() {
		let attempts = Cell::new(0);
		let value = retry(|| {
			attempts.set(attempts.get() + 1);
			if attempts.get() < 3 { Err(io::Error::from(ErrorKind::PermissionDenied)) } else { Ok(7) }
		})
		.unwrap();
		assert_eq!((value, attempts.get()), (7, 3));
	}

	#[test]
	fn other_failures_are_not_retried() {
		let attempts = Cell::new(0);
		let error = retry::<()>(|| {
			attempts.set(attempts.get() + 1);
			Err(io::Error::from(ErrorKind::NotFound))
		})
		.unwrap_err();
		assert_eq!((error.kind(), attempts.get()), (ErrorKind::NotFound, 1));
	}
}
