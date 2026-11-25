//! Biome definitions by id, read from the worldgen data of the server jar and of mods.

use std::collections::HashMap;
use std::path::Path;

use super::Biome;
use crate::error::Result;
use crate::jar;

#[derive(Default)]
pub struct BiomeRegistry {
	biomes: HashMap<String, Biome>,
}

impl BiomeRegistry {
	/// Adds the biomes of a jar. A later jar replaces an earlier definition of the same id, as mods override vanilla.
	pub fn add_jar(&mut self, path: &Path) -> Result<()> {
		jar::visit_entries(path, |name| biome_id(name).is_some(), &mut |name: &str, bytes: Vec<u8>| {
			if let (Some(id), Ok(biome)) = (biome_id(name), Biome::from_json(&bytes)) {
				self.biomes.insert(id, biome);
			}
		})
	}

	/// Unknown biomes take plains.
	pub fn get(&self, id: &str) -> &Biome {
		self.biomes.get(id).unwrap_or(&Biome::PLAINS)
	}
}

/// `data/<namespace>/worldgen/biome/<path>.json` defines the biome `<namespace>:<path>`.
fn biome_id(name: &str) -> Option<String> {
	let (namespace, relative) = name.strip_prefix("data/")?.split_once('/')?;
	let path = relative.strip_prefix("worldgen/biome/")?.strip_suffix(".json")?;
	Some(format!("{namespace}:{path}"))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_support::write_jar;

	#[test]
	fn reads_biomes_from_jars_and_lets_later_jars_override() {
		let dir = tempfile::tempdir().unwrap();
		let vanilla = dir.path().join("server.jar");
		let modded = dir.path().join("mod.jar");
		write_jar(
			&vanilla,
			&[
				("data/minecraft/worldgen/biome/swamp.json", br#"{"temperature": 0.8}"#),
				("data/minecraft/tags/worldgen/biome/is_forest.json", b"{}"),
			],
		);
		write_jar(
			&modded,
			&[
				("data/minecraft/worldgen/biome/swamp.json", br#"{"temperature": 0.1}"#),
				("data/mymod/worldgen/biome/glade.json", br#"{"temperature": 0.3}"#),
			],
		);
		let mut registry = BiomeRegistry::default();
		registry.add_jar(&vanilla).unwrap();
		registry.add_jar(&modded).unwrap();
		assert!((registry.get("minecraft:swamp").temperature - 0.1).abs() < 1e-6);
		assert!((registry.get("mymod:glade").temperature - 0.3).abs() < 1e-6);
		assert_eq!(registry.get("minecraft:is_forest"), &Biome::PLAINS);
	}

	#[test]
	fn unknown_biomes_take_plains() {
		assert_eq!(BiomeRegistry::default().get("minecraft:desert"), &Biome::PLAINS);
	}
}
