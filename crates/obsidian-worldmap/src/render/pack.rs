//! The assets, biomes and block defaults that a set of renders shares, and the renderer that draws regions with them.

use std::path::{Path, PathBuf};

use image::RgbaImage;

use crate::assets::AssetStore;
use crate::atlas::{Atlas, block_textures};
use crate::biome::BiomeRegistry;
use crate::chunk::{BlockDefaults, BlockState};
use crate::models::Blockstate;
use crate::structure::{self, BiomeEntry, BiomeTable, BlockTable, Lighting, StateTable, Vocabulary};
use crate::colour::{BlockColours, SurfaceCache};
use crate::error::Result;
use crate::region::RegionFile;
use crate::render::{RegionManifest, render_region};
use crate::tiles::detail;

/// Where a pack's data comes from. Mods load after the client and server jars, so they override both, as in the game.
pub struct PackSources<'a> {
	pub client_jar: &'a Path,
	pub server_jar: &'a Path,
	pub mod_jars: &'a [PathBuf],
	pub blocks_report: &'a Path,
}

/// Loaded once per build: the assets, biome definitions and block defaults every region is drawn with.
pub struct Pack {
	assets: AssetStore,
	biomes: BiomeRegistry,
	defaults: BlockDefaults,
}

impl Pack {
	pub fn load(sources: &PackSources<'_>) -> Result<Self> {
		let mut assets = AssetStore::default();
		assets.add_jar(sources.client_jar)?;
		let mut biomes = BiomeRegistry::default();
		biomes.add_jar(sources.server_jar)?;
		for jar in sources.mod_jars {
			assets.add_jar(jar)?;
			biomes.add_jar(jar)?;
		}
		let mut defaults = BlockDefaults::parse(&std::fs::read(sources.blocks_report)?)?;
		infer_mod_defaults(&assets, &mut defaults);
		Ok(Self { assets, biomes, defaults })
	}

	/// One textured detail tile of a region. `cache` keeps the textures decoded so far, so tiles drawn in a row reuse them.
	pub fn render_detail_tile(&self, cache: &mut SurfaceCache, region: &RegionFile, tile_x: usize, tile_z: usize) -> Result<RgbaImage> {
		let mut colours = BlockColours::with_cache(&self.assets, &self.biomes, std::mem::take(cache));
		let image = detail::render_detail_tile(region, tile_x, tile_z, &self.defaults, &mut colours);
		*cache = colours.into_cache();
		image
	}

	pub fn block_table(&self, states: &[BlockState], atlas: &Atlas, lighting: &Lighting) -> BlockTable {
		structure::block_table(&self.assets, states, atlas, lighting)
	}

	pub fn biome_table(&self, names: &[String]) -> Vec<BiomeEntry> {
		structure::biome_table(&self.assets, &self.biomes, names)
	}

	pub fn region_vocabulary(&self, region: &Path) -> Result<Vocabulary> {
		structure::vocabulary(region, &self.defaults)
	}

	pub fn write_region_pack(&self, region: &Path, structure_dir: &Path, states: &StateTable, biomes: &BiomeTable) -> Result<usize> {
		structure::write_region_pack(region, structure_dir, &self.defaults, states, biomes)
	}
}

/// Mod blocks are missing from the vanilla report, so their defaults are inferred from their blockstates.
fn infer_mod_defaults(assets: &AssetStore, defaults: &mut BlockDefaults) {
	for (name, json) in assets.blockstates() {
		if defaults.knows(&name) {
			continue;
		}
		if let Some(properties) = Blockstate::parse(json).ok().and_then(|blockstate| blockstate.default_properties()) {
			defaults.insert_inferred(name, properties);
		}
	}
}

/// Draws the regions of one pack. The atlas is fixed by the pack, so every region's UVs agree with each other.
pub struct Renderer<'a> {
	pack: &'a Pack,
	colours: BlockColours<'a>,
	atlas: Atlas,
}

impl<'a> Renderer<'a> {
	pub fn new(pack: &'a Pack) -> Self {
		Self {
			pack,
			colours: BlockColours::new(&pack.assets, &pack.biomes),
			atlas: {
				let textures = block_textures(&pack.assets);
				Atlas::build(&pack.assets, textures.ids.iter().map(String::as_str), &textures.translucent, &textures.large)
			},
		}
	}

	pub fn atlas(&self) -> &Atlas {
		&self.atlas
	}

	pub fn render_region(&mut self, region: &Path, dimension_dir: &Path) -> Result<RegionManifest> {
		render_region(region, dimension_dir, &self.pack.defaults, &mut self.colours)
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use super::*;
	use crate::test_support::write_jar;

	#[test]
	fn mod_blocks_the_report_lacks_get_defaults_from_their_blockstates() {
		let dir = tempfile::tempdir().unwrap();
		let jar = dir.path().join("mod.jar");
		write_jar(
			&jar,
			&[(
				"assets/techreborn/blockstates/alloy_smelter.json",
				br#"{"variants": {"active=false,facing=north": {"model": "techreborn:block/a"}, "active=true,facing=north": {"model": "techreborn:block/b"}}}"#,
			)],
		);
		let mut assets = AssetStore::default();
		assets.add_jar(&jar).unwrap();
		let mut defaults = BlockDefaults::parse(b"{}").unwrap();
		infer_mod_defaults(&assets, &mut defaults);
		let mut bare = BlockState {
			name: "techreborn:alloy_smelter".into(),
			properties: BTreeMap::new(),
		};
		defaults.fill(&mut bare);
		assert_eq!(bare.properties.get("active").map(String::as_str), Some("false"));
	}
}
