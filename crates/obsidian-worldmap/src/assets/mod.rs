//! Block, model and texture entries from Minecraft's client jar and from mod jars.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::Result;

const FOLDERS: [&str; 3] = ["blockstates/", "models/", "textures/"];

/// Every `assets/<namespace>/{blockstates,models,textures}` entry, keyed by namespace and the path inside it.
#[derive(Default)]
pub struct AssetStore {
	entries: HashMap<(String, String), Vec<u8>>,
}

impl AssetStore {
	/// Adds a jar's entries. Entries from later jars replace earlier ones, which is how mods override vanilla.
	pub fn add_jar(&mut self, path: &Path) -> Result<()> {
		let mut archive = zip::ZipArchive::new(File::open(path)?)?;
		for index in 0..archive.len() {
			let mut file = archive.by_index(index)?;
			if file.is_dir() {
				continue;
			}
			let Ok(name) = file.name() else { continue };
			let Some((namespace, relative)) = split_asset_name(&name) else { continue };
			let mut bytes = Vec::with_capacity(file.size() as usize);
			file.read_to_end(&mut bytes)?;
			self.insert(namespace, relative, bytes);
		}
		Ok(())
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
}

fn split_asset_name(name: &str) -> Option<(String, String)> {
	let rest = name.strip_prefix("assets/")?;
	let (namespace, relative) = rest.split_once('/')?;
	let wanted = FOLDERS.iter().any(|folder| relative.starts_with(folder))
		&& (relative.ends_with(".json") || relative.ends_with(".png"));
	wanted.then(|| (namespace.to_owned(), relative.to_owned()))
}

#[cfg(test)]
mod tests {
	use std::io::Write;

	use super::*;

	fn write_jar(path: &Path, entries: &[(&str, &[u8])]) {
		let mut writer = zip::ZipWriter::new(File::create(path).unwrap());
		for (name, bytes) in entries {
			writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
			writer.write_all(bytes).unwrap();
		}
		writer.finish().unwrap();
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
}
