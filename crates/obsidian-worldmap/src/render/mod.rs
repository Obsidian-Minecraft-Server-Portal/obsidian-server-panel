//! Renders one region into its top-down tile, and records what was written in a region manifest. Block structure is written
//! separately by the structure module; the browser meshes it.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::atomic;
use crate::chunk::{BlockDefaults, Chunk, is_invisible, load_chunks, read_chunk};
use crate::colour::BlockColours;
use crate::error::Result;
use crate::region::{CHUNKS_PER_SIDE, RegionFile};
use crate::structure::region_pack_path;
use crate::tiles::{encode_png, render_tile};
use crate::world::BlockWorld;

mod pack;

pub use pack::{Pack, PackSources, Renderer};

/// Offsets of the four regions that share an edge with this one.
const NEIGHBOURS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const LAST_CHUNK: usize = CHUNKS_PER_SIDE - 1;

/// What one region wrote, stored as `regions/r.X.Z.json` in its dimension folder. Paths are relative to that folder.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegionManifest {
	pub x: i32,
	pub z: i32,
	pub tile: String,
	/// Every section holding a block other than air, in order.
	pub sections: Vec<SectionEntry>,
	/// Unix milliseconds when the region was drawn. Viewers key cached files by it, so a redraw replaces them.
	pub rendered_at: i64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SectionEntry {
	pub x: i32,
	pub y: i32,
	pub z: i32,
	pub file: String,
}

/// Writes the top-down tile and the manifest of the region at `path` under `dimension_dir`.
///
/// Tiles need the blocks along each shared edge for slope shading, so the chunks on the facing edges of the neighbouring
/// regions are loaded for lookups but not drawn. A missing neighbour leaves its edge flat, the same as an unknown block.
pub fn render_region(path: &Path, dimension_dir: &Path, defaults: &BlockDefaults, colours: &mut BlockColours<'_>) -> Result<RegionManifest> {
	let region = RegionFile::open(path)?;
	let (region_x, region_z) = region.region_coords();
	let mut chunks = load_chunks(&region);
	chunks.extend(neighbour_edges(path, region_x, region_z));
	let world = BlockWorld::new(chunks, defaults);

	let stem = format!("r.{region_x}.{region_z}");
	let tile = format!("tiles/{stem}.png");
	write(&dimension_dir.join(&tile), &encode_png(&render_tile(&world, region_x, region_z, colours))?)?;
	let detail = dimension_dir.join("detail").join(&stem);
	if detail.exists() {
		fs::remove_dir_all(&detail)?;
	}
	let legacy_meshes = dimension_dir.join("sections").join(&stem);
	if legacy_meshes.exists() {
		fs::remove_dir_all(&legacy_meshes)?;
	}

	let pack_file = region_pack_path((region_x, region_z));
	let pack_file = pack_file.as_str();
	let mut sections: Vec<SectionEntry> = world
		.chunks()
		.filter(|chunk| owns(chunk, region_x, region_z))
		.flat_map(|chunk| {
			chunk
				.sections()
				.iter()
				.filter(|section| !section.palette().iter().all(|state| is_invisible(&state.name)))
				.map(move |section| SectionEntry {
					x: chunk.x,
					y: section.y(),
					z: chunk.z,
					file: pack_file.to_owned(),
				})
		})
		.collect();
	sections.sort_by_key(|entry| (entry.x, entry.y, entry.z));

	let manifest = RegionManifest {
		x: region_x,
		z: region_z,
		tile,
		sections,
		rendered_at: rendered_at(),
	};
	let json = serde_json::to_vec(&manifest).map_err(std::io::Error::other)?;
	write(&dimension_dir.join("regions").join(format!("{stem}.json")), &json)?;
	Ok(manifest)
}

fn rendered_at() -> i64 {
	SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as i64)
}

fn owns(chunk: &Chunk, region_x: i32, region_z: i32) -> bool {
	let size = CHUNKS_PER_SIDE as i32;
	chunk.x.div_euclid(size) == region_x && chunk.z.div_euclid(size) == region_z
}

/// The chunks along the edges of the neighbouring regions that face this one. Unreadable or missing regions add nothing.
fn neighbour_edges(path: &Path, region_x: i32, region_z: i32) -> Vec<Chunk> {
	let Some(directory) = path.parent() else { return Vec::new() };
	let mut chunks = Vec::new();
	for (dx, dz) in NEIGHBOURS {
		let file = directory.join(format!("r.{}.{}.mca", region_x + dx, region_z + dz));
		let Ok(neighbour) = RegionFile::open(&file) else { continue };
		for along in 0..CHUNKS_PER_SIDE {
			let (local_x, local_z) = match (dx, dz) {
				(-1, _) => (LAST_CHUNK, along),
				(1, _) => (0, along),
				(_, -1) => (along, LAST_CHUNK),
				_ => (along, 0),
			};
			chunks.extend(read_chunk(&neighbour, local_x, local_z));
		}
	}
	chunks
}

fn write(path: &Path, bytes: &[u8]) -> Result<()> {
	atomic::write(path, bytes)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::biome::BiomeRegistry;
	use crate::test_support::{chunk_nbt, region_bytes, region_records, stone_assets, zlib};

	fn solid_chunk(x: i32, z: i32) -> Vec<u8> {
		let blocks: Vec<((usize, usize, usize), &str)> = (0..16)
			.flat_map(|y| (0..16).flat_map(move |z| (0..16).map(move |x| ((x, y, z), "minecraft:stone"))))
			.collect();
		chunk_nbt(x, z, 3955, &blocks)
	}

	#[test]
	fn manifest_lists_the_sections_holding_blocks_under_their_structure_paths() {
		let dir = tempfile::tempdir().unwrap();
		let region = dir.path().join("r.0.0.mca");
		fs::write(&region, region_bytes(&[(0, 0, solid_chunk(0, 0))])).unwrap();
		let assets = stone_assets();
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let out = dir.path().join("out");
		let manifest = render_region(&region, &out, &BlockDefaults::default(), &mut colours).unwrap();
		assert_eq!(
			manifest.sections,
			vec![SectionEntry {
				x: 0,
				y: 0,
				z: 0,
				file: "structure/regions/r.0.0.obsr".to_owned(),
			}]
		);
		assert!(out.join(&manifest.tile).exists());
	}

	#[test]
	fn a_region_with_a_damaged_chunk_still_renders_its_readable_chunks() {
		let dir = tempfile::tempdir().unwrap();
		let region = dir.path().join("r.0.0.mca");
		fs::write(&region, region_records(&[(0, 0, zlib(&solid_chunk(0, 0))), (1, 0, vec![0xde, 0xad, 0xbe, 0xef])])).unwrap();
		let assets = stone_assets();
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let out = dir.path().join("out");
		let manifest = render_region(&region, &out, &BlockDefaults::default(), &mut colours).unwrap();
		assert_eq!(manifest.sections.len(), 1);
		assert!(out.join(&manifest.tile).exists());
	}
}
