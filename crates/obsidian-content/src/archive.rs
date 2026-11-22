use std::collections::BTreeSet;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

use obsidian_common::{CommonError, Result};
use zip::ZipArchive;

const METADATA_LIMIT: u64 = 4 << 20;
const PACK_FOLDERS: [&str; 12] = [
	"mods",
	"plugins",
	"config",
	"defaultconfigs",
	"kubejs",
	"scripts",
	"global_packs",
	"datapacks",
	"resourcepacks",
	"shaderpacks",
	"overrides",
	"server-overrides",
];
const LOADER_FOLDERS: [&str; 4] = ["libraries", "versions", ".fabric", ".quilt"];

pub type Archive<'a> = ZipArchive<Cursor<&'a [u8]>>;

pub fn open(bytes: &[u8]) -> Result<Archive<'_>> {
	ZipArchive::new(Cursor::new(bytes)).map_err(|error| CommonError::Invalid(format!("not a valid zip archive: {error}")))
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
	extract_prefixes_where(archive_bytes, prefixes, directory, |_| true)
}

pub fn extract_prefixes_where(
	archive_bytes: &[u8],
	prefixes: &[String],
	directory: &Path,
	keep: impl Fn(&str) -> bool,
) -> Result<()> {
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
			if relative.as_os_str().is_empty() || !keep(&pack_path(relative)) {
				continue;
			}
			let is_dir = entry.is_dir();
			write_entry(&mut entry, is_dir, &directory.join(relative))?;
		}
	}
	Ok(())
}

pub fn extract_server_pack(archive_bytes: &[u8], directory: &Path) -> Result<Vec<String>> {
	let mut archive = open(archive_bytes)?;
	let root = pack_root(&mut archive);
	let mut written = Vec::new();
	for index in 0..archive.len() {
		let mut entry = archive
			.by_index(index)
			.map_err(|error| CommonError::Invalid(format!("corrupt archive entry: {error}")))?;
		let Some(name) = entry.enclosed_name() else {
			continue;
		};
		let relative = match &root {
			Some(root) => match name.strip_prefix(root) {
				Ok(rest) => rest.to_path_buf(),
				Err(_) => continue,
			},
			None => name,
		};
		if relative.as_os_str().is_empty() || is_loader_artifact(&relative) {
			continue;
		}
		let is_dir = entry.is_dir();
		write_entry(&mut entry, is_dir, &directory.join(&relative))?;
		if !is_dir {
			written.push(pack_path(&relative));
		}
	}
	Ok(written)
}

pub fn override_folders(archive_bytes: &[u8], prefixes: &[String]) -> Result<BTreeSet<String>> {
	let mut archive = open(archive_bytes)?;
	let mut folders = BTreeSet::new();
	for prefix in prefixes {
		for index in 0..archive.len() {
			let Ok(entry) = archive.by_index(index) else {
				continue;
			};
			let Some(name) = entry.enclosed_name() else {
				continue;
			};
			let Ok(relative) = name.strip_prefix(prefix) else {
				continue;
			};
			let mut parts = relative.components();
			if let (Some(first), Some(_)) = (parts.next(), parts.next()) {
				folders.insert(first.as_os_str().to_string_lossy().into_owned());
			}
		}
	}
	Ok(folders)
}

fn write_entry(entry: &mut impl Read, is_dir: bool, target: &Path) -> Result<()> {
	if is_dir {
		std::fs::create_dir_all(target)?;
		return Ok(());
	}
	if let Some(parent) = target.parent() {
		std::fs::create_dir_all(parent)?;
	}
	std::io::copy(entry, &mut std::fs::File::create(target)?)?;
	Ok(())
}

fn pack_root(archive: &mut Archive<'_>) -> Option<String> {
	let names: Vec<PathBuf> = (0..archive.len())
		.filter_map(|index| archive.by_index(index).ok()?.enclosed_name())
		.collect();
	let roots: BTreeSet<String> = names
		.iter()
		.filter_map(|name| name.components().next())
		.map(|component| component.as_os_str().to_string_lossy().into_owned())
		.collect();
	let root = roots.iter().next()?.clone();
	let single = roots.len() == 1
		&& !PACK_FOLDERS.contains(&root.as_str())
		&& names.iter().all(|name| name.components().count() > 1);
	single.then_some(root)
}

fn is_loader_artifact(relative: &Path) -> bool {
	let parts: Vec<String> = relative
		.components()
		.map(|component| component.as_os_str().to_string_lossy().into_owned())
		.collect();
	match parts.as_slice() {
		[first, ..] if LOADER_FOLDERS.contains(&first.as_str()) => true,
		[name] => name.ends_with(".jar") || matches!(name.as_str(), "manifest.json" | "modlist.html"),
		_ => false,
	}
}

fn pack_path(path: &Path) -> String {
	path.components()
		.filter_map(|component| match component {
			Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
			_ => None,
		})
		.collect::<Vec<_>>()
		.join("/")
}
