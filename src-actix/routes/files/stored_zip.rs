//! Zip writers for file-manager archives. On-disk archives are stored, with sizes in their headers.
//! Streamed downloads are deflated at level 1; see `write_stream`.

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufWriter, Read, Seek, Write};
use std::path::{Component, Path, PathBuf};

use obsidian_common::{CommonError, Result};
use walkdir::WalkDir;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

const BUFFER: usize = 256 * 1024;
const LARGE_FILE: u64 = 4 * 1024 * 1024 * 1024;
/// Level 0 would store the data, but zip 9.8.1 rejects it, so this is the lowest level it accepts.
const STREAM_LEVEL: i64 = 1;

#[derive(Debug)]
pub struct ZipEntry {
	pub path: PathBuf,
	/// Name inside the archive: relative to the base folder, with `/` separators.
	pub name: String,
	pub is_dir: bool,
	pub size: u64,
}

#[derive(Debug, Default)]
pub struct Plan {
	pub entries: Vec<ZipEntry>,
	/// Total size of the file entries, the denominator for progress.
	pub bytes: u64,
}

/// Symlinks are skipped so an archive cannot carry links out of the instance. `excluded` paths are
/// skipped so an archive never contains itself or its partial copy.
pub fn collect(sources: &[PathBuf], base: &Path, excluded: &[PathBuf]) -> Result<Plan> {
	let mut plan = Plan::default();
	let mut names = HashSet::new();
	for source in sources {
		for entry in WalkDir::new(source).follow_links(false) {
			let entry = entry.map_err(|error| CommonError::Other(error.to_string()))?;
			let (path, kind) = (entry.path(), entry.file_type());
			if kind.is_symlink() || excluded.iter().any(|skip| skip.as_path() == path) {
				continue;
			}
			let relative = path
				.strip_prefix(base)
				.map_err(|_| CommonError::Invalid(format!("`{}` is outside the archive folder", path.display())))?;
			let name = zip_name(relative);
			if name.is_empty() || !names.insert(name.clone()) {
				continue;
			}
			let size = if kind.is_dir() {
				0
			} else {
				entry.metadata().map_err(|error| CommonError::Other(error.to_string()))?.len()
			};
			plan.bytes += size;
			plan.entries.push(ZipEntry { path: path.to_path_buf(), name, is_dir: kind.is_dir(), size });
		}
	}
	Ok(plan)
}

/// Stored members, with sizes and CRC patched into each local header once the member is written, so
/// no data descriptor follows the data.
pub fn write_file<W: Write + Seek>(out: W, entries: &[ZipEntry], on_read: impl FnMut(u64, &str)) -> Result<W> {
	let mut zip = zip::ZipWriter::new(BufWriter::with_capacity(BUFFER, out));
	write_members(&mut zip, entries, |size| stored().large_file(size >= LARGE_FILE), on_read)?;
	let buffered = zip.finish().map_err(zip_err)?;
	buffered.into_inner().map_err(|error| CommonError::Io(error.into_error()))
}

/// Streamed downloads cannot seek, so each member's sizes follow its data in a data descriptor.
/// Members are deflated at level 1, because zip 9.8.1 rejects level 0 and streaming readers such as
/// Java's ZipInputStream reject stored members with data descriptors. Level 1 costs about 85 times the
/// CPU of storing on incompressible data.
pub fn write_stream<W: Write>(out: W, entries: &[ZipEntry], on_read: impl FnMut(u64, &str)) -> Result<W> {
	let mut zip = zip::ZipWriter::new_stream(BufWriter::with_capacity(BUFFER, out));
	write_members(&mut zip, entries, |size| deflated().large_file(size >= LARGE_FILE), on_read)?;
	let buffered = zip.finish().map_err(zip_err)?.into_inner();
	buffered.into_inner().map_err(|error| CommonError::Io(error.into_error()))
}

fn write_members<W: Write + Seek>(
	zip: &mut zip::ZipWriter<W>,
	entries: &[ZipEntry],
	member_options: impl Fn(u64) -> SimpleFileOptions,
	mut on_read: impl FnMut(u64, &str),
) -> Result<()> {
	let mut buffer = vec![0; BUFFER];
	for entry in entries {
		if entry.is_dir {
			zip.add_directory(format!("{}/", entry.name), stored()).map_err(zip_err)?;
			continue;
		}
		zip.start_file(entry.name.as_str(), member_options(entry.size)).map_err(zip_err)?;
		copy_member(zip, &entry.path, &entry.name, &mut buffer, &mut on_read)?;
	}
	Ok(())
}

/// `on_read` gets the size of every chunk, so progress follows the bytes read from disk.
fn copy_member(
	out: &mut impl Write,
	path: &Path,
	name: &str,
	buffer: &mut [u8],
	on_read: &mut impl FnMut(u64, &str),
) -> Result<()> {
	let mut source = File::open(path)?;
	loop {
		let read = match source.read(buffer) {
			Ok(0) => return Ok(()),
			Ok(read) => read,
			Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
			Err(error) => return Err(error.into()),
		};
		out.write_all(&buffer[..read])?;
		on_read(read as u64, name);
	}
}

fn stored() -> SimpleFileOptions {
	SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
}

fn deflated() -> SimpleFileOptions {
	SimpleFileOptions::default()
		.compression_method(CompressionMethod::Deflated)
		.compression_level(Some(STREAM_LEVEL))
}

fn zip_name(relative: &Path) -> String {
	relative
		.components()
		.filter_map(|component| match component {
			Component::Normal(part) => Some(part.to_string_lossy()),
			_ => None,
		})
		.collect::<Vec<_>>()
		.join("/")
}

fn zip_err(error: zip::result::ZipError) -> CommonError {
	match error {
		zip::result::ZipError::Io(io) => CommonError::Io(io),
		other => CommonError::Other(other.to_string()),
	}
}

#[cfg(test)]
mod tests {
	use std::fs;
	use std::io::{Cursor, Read};

	use super::*;

	const DATA_DESCRIPTOR: u16 = 1 << 3;

	fn sample_plan() -> (tempfile::TempDir, Plan) {
		let dir = tempfile::tempdir().unwrap();
		let base = dir.path().to_path_buf();
		fs::create_dir_all(base.join("sub/deep")).unwrap();
		fs::write(base.join("a.txt"), b"alpha").unwrap();
		fs::write(base.join("sub/deep/b.txt"), b"bravo").unwrap();
		let plan = collect(&[base.join("a.txt"), base.join("sub")], &base, &[]).unwrap();
		(dir, plan)
	}

	struct LocalHeader {
		method: u16,
		flags: u16,
		compressed: u32,
		uncompressed: u32,
	}

	/// The first local header, which belongs to `a.txt`.
	fn first_local_header(bytes: &[u8]) -> LocalHeader {
		let u16_at = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
		let u32_at = |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
		LocalHeader { method: u16_at(8), flags: u16_at(6), compressed: u32_at(18), uncompressed: u32_at(22) }
	}

	fn assert_round_trip(bytes: Vec<u8>, method: CompressionMethod) {
		let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
		let mut names: Vec<String> = archive.file_names().map(|name| name.unwrap().into_owned()).collect();
		names.sort();
		assert_eq!(names, ["a.txt", "sub/", "sub/deep/", "sub/deep/b.txt"]);
		let mut member = archive.by_name("sub/deep/b.txt").unwrap();
		assert_eq!(member.compression(), method);
		let mut text = String::new();
		member.read_to_string(&mut text).unwrap();
		assert_eq!(text, "bravo");
	}

	#[test]
	fn file_output_is_stored_with_sizes_in_local_headers() {
		let (_dir, plan) = sample_plan();
		let mut reported = 0;
		let bytes = write_file(Cursor::new(Vec::new()), &plan.entries, |read, _| reported += read).unwrap().into_inner();

		assert_eq!(reported, plan.bytes);
		let header = first_local_header(&bytes);
		assert_eq!(header.method, 0);
		assert_eq!(header.flags & DATA_DESCRIPTOR, 0);
		assert_eq!((header.compressed, header.uncompressed), (5, 5));
		assert_round_trip(bytes, CompressionMethod::Stored);
	}

	#[test]
	fn streamed_output_is_deflated_with_data_descriptors() {
		let (_dir, plan) = sample_plan();
		let bytes = write_stream(Cursor::new(Vec::new()), &plan.entries, |_, _| {}).unwrap().into_inner();

		let header = first_local_header(&bytes);
		assert_eq!(header.method, 8);
		assert_ne!(header.flags & DATA_DESCRIPTOR, 0);
		assert_eq!((header.compressed, header.uncompressed), (0, 0));
		assert_round_trip(bytes, CompressionMethod::Deflated);
	}

	#[test]
	fn excluded_paths_are_left_out() {
		let dir = tempfile::tempdir().unwrap();
		let base = dir.path();
		fs::write(base.join("keep.txt"), b"kept").unwrap();
		fs::write(base.join("out.zip.part"), b"partial").unwrap();
		let plan = collect(&[base.to_path_buf()], base, &[base.join("out.zip.part")]).unwrap();
		let names: Vec<&str> = plan.entries.iter().map(|entry| entry.name.as_str()).collect();
		assert_eq!(names, ["keep.txt"]);
	}
}
