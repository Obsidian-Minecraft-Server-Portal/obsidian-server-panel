use std::io::{Cursor, Read};
use std::path::Path;

use obsidian_common::{CommonError, Result};
use zip::ZipArchive;

const METADATA_LIMIT: u64 = 4 << 20;

pub type Archive<'a> = ZipArchive<Cursor<&'a [u8]>>;

pub fn open(bytes: &[u8]) -> Result<Archive<'_>> {
	ZipArchive::new(Cursor::new(bytes))
		.map_err(|error| CommonError::Invalid(format!("not a valid zip archive: {error}")))
}

pub fn read(archive: &mut Archive<'_>, name: &str, limit: u64) -> Option<Vec<u8>> {
	let mut entry = archive.by_name(name).ok()?;
	if entry.size() > limit {
		return None;
	}
	let mut buffer = Vec::with_capacity(entry.size() as usize);
	entry.read_to_end(&mut buffer).ok()?;
	Some(buffer)
}

pub fn read_text(archive: &mut Archive<'_>, name: &str) -> Option<String> {
	String::from_utf8(read(archive, name, METADATA_LIMIT)?).ok()
}

pub fn extract_prefixes(archive_bytes: &[u8], prefixes: &[String], directory: &Path) -> Result<()> {
	let mut archive = open(archive_bytes)?;
	for prefix in prefixes {
		for index in 0..archive.len() {
			let mut entry = archive
				.by_index(index)
				.map_err(|error| CommonError::Invalid(format!("corrupt archive entry: {error}")))?;
			let Some(name) = entry.enclosed_name() else {
				continue;
			};
			let Ok(relative) = name.strip_prefix(prefix) else {
				continue;
			};
			if relative.as_os_str().is_empty() {
				continue;
			}
			let target = directory.join(relative);
			if entry.is_dir() {
				std::fs::create_dir_all(&target)?;
				continue;
			}
			if let Some(parent) = target.parent() {
				std::fs::create_dir_all(parent)?;
			}
			std::io::copy(&mut entry, &mut std::fs::File::create(&target)?)?;
		}
	}
	Ok(())
}
