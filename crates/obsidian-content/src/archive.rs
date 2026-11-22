use std::io::{Cursor, Read};

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
