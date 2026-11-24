//! Block, model and texture entries from Minecraft's client jar and from mod jars, including jars nested inside a mod.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use crate::error::Result;

const FOLDERS: [&str; 3] = ["blockstates/", "models/", "textures/"];
const NESTED_FOLDERS: [&str; 2] = ["META-INF/jars/", "META-INF/jarjar/"];
const MAX_NESTING: usize = 3;

/// Every `assets/<namespace>/{blockstates,models,textures}` entry, keyed by namespace and the path inside it.
#[derive(Default)]
pub struct AssetStore {
	entries: HashMap<(String, String), Vec<u8>>,
}

impl AssetStore {
	/// Adds a jar's entries. Entries from later jars replace earlier ones, which is how mods override vanilla.
	pub fn add_jar(&mut self, path: &Path) -> Result<()> {
		let archive = zip::ZipArchive::new(File::open(path)?)?;
		self.add_archive(archive, 0)
	}

	/// `relative` is a path inside the namespace, such as `models/block/stone.json`.
	pub fn get(&self, namespace: &str, relative: &str) -> Option<&[u8]> {
		self.entries
			.get(&(namespace.to_owned(), relative.to_owned()))
			.map(Vec::as_slice)
	}

	pub(crate) fn insert(&mut self, namespace: String, relative: String, bytes: Vec<u8>) {
		self.entries.insert((namespace, relative), bytes);
	}

	fn add_archive<R: Read + Seek>(&mut self, mut archive: zip::ZipArchive<R>, depth: usize) -> Result<()> {
		for index in 0..archive.len() {
			let mut file = archive.by_index(index)?;
			if file.is_dir() {
				continue;
			}
			let Ok(name) = file.name().map(|name| name.into_owned()) else { continue };
			if let Some((namespace, relative)) = split_asset_name(&name) {
				let mut bytes = Vec::with_capacity(file.size() as usize);
				file.read_to_end(&mut bytes)?;
				self.insert(namespace, relative, bytes);
			} else if depth < MAX_NESTING && is_nested_jar(&name) {
				let mut bytes = Vec::with_capacity(file.size() as usize);
				file.read_to_end(&mut bytes)?;
				if let Ok(nested) = zip::ZipArchive::new(Cursor::new(bytes)) {
					self.add_archive(nested, depth + 1)?;
				}
			}
		}
		Ok(())
	}
}

fn split_asset_name(name: &str) -> Option<(String, String)> {
	let rest = name.strip_prefix("assets/")?;
	let (namespace, relative) = rest.split_once('/')?;
	let wanted = FOLDERS.iter().any(|folder| relative.starts_with(folder))
		&& (relative.ends_with(".json") || relative.ends_with(".png"));
	wanted.then(|| (namespace.to_owned(), relative.to_owned()))
}

fn is_nested_jar(name: &str) -> bool {
	name.ends_with(".jar") && NESTED_FOLDERS.iter().any(|folder| name.starts_with(folder))
}

#[cfg(test)]
mod tests {
	use std::io::Write;

	use super::*;

	fn write_jar(path: &Path, entries: &[(&str, &[u8])]) {
		std::fs::write(path, zip_bytes(entries)).unwrap();
	}

	fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
		let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
		for (name, bytes) in entries {
			writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
			writer.write_all(bytes).unwrap();
		}
		writer.finish().unwrap().into_inner()
	}

	#[test]
	fn keeps_block_model_and_texture_entries_only() {
		let dir = tempfile::tempdir().unwrap();
		let jar = dir.path().join("client.jar");
		write_jar(
			&jar,
			&[
				("assets/minecraft/models/block/stone.json", b"{}"),
				("assets/minecraft/textures/block/stone.png", b"png"),
				("assets/minecraft/lang/en_us.json", b"{}"),
				("data/minecraft/worldgen/biome/plains.json", b"{}"),
			],
		);
		let mut store = AssetStore::default();
		store.add_jar(&jar).unwrap();
		assert!(store.get("minecraft", "models/block/stone.json").is_some());
		assert!(store.get("minecraft", "textures/block/stone.png").is_some());
		assert!(store.get("minecraft", "lang/en_us.json").is_none());
	}

	#[test]
	fn later_jars_override_earlier_entries() {
		let dir = tempfile::tempdir().unwrap();
		let vanilla = dir.path().join("vanilla.jar");
		let modded = dir.path().join("mod.jar");
		write_jar(&vanilla, &[("assets/minecraft/models/block/stone.json", b"vanilla")]);
		write_jar(&modded, &[("assets/minecraft/models/block/stone.json", b"modded")]);
		let mut store = AssetStore::default();
		store.add_jar(&vanilla).unwrap();
		store.add_jar(&modded).unwrap();
		assert_eq!(store.get("minecraft", "models/block/stone.json"), Some(&b"modded"[..]));
	}

	#[test]
	fn reads_jars_nested_under_fabric_and_neoforge_folders() {
		let dir = tempfile::tempdir().unwrap();
		let fabric_inner = zip_bytes(&[("assets/fabricmod/models/block/a.json", b"{}")]);
		let forge_inner = zip_bytes(&[("assets/forgemod/textures/block/b.png", b"png")]);
		let outer = dir.path().join("mod.jar");
		write_jar(
			&outer,
			&[
				("META-INF/jars/fabric-inner.jar", &fabric_inner),
				("META-INF/jarjar/forge-inner.jar", &forge_inner),
			],
		);
		let mut store = AssetStore::default();
		store.add_jar(&outer).unwrap();
		assert!(store.get("fabricmod", "models/block/a.json").is_some());
		assert!(store.get("forgemod", "textures/block/b.png").is_some());
	}

	#[test]
	fn stops_following_nested_jars_past_the_depth_limit() {
		let mut inner = zip_bytes(&[("assets/deep/models/block/x.json", b"{}")]);
		for level in 0..MAX_NESTING {
			inner = zip_bytes(&[(&format!("META-INF/jars/level{level}.jar"), &inner)]);
		}
		let dir = tempfile::tempdir().unwrap();
		let outer = dir.path().join("mod.jar");
		write_jar(&outer, &[("META-INF/jars/outer.jar", &inner)]);
		let mut store = AssetStore::default();
		store.add_jar(&outer).unwrap();
		assert!(store.get("deep", "models/block/x.json").is_none());
	}
}
