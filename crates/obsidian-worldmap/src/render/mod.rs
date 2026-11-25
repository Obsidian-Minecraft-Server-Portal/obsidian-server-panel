//! Renders one region into its top-down tile and section meshes, and records what was written in a region manifest.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::chunk::{BlockDefaults, Chunk};
use crate::colour::BlockColours;
use crate::error::Result;
use crate::mesh::{BlockWorld, MeshContext, format};
use crate::region::{CHUNKS_PER_SIDE, RegionFile};
use crate::tiles::{encode_png, render_tile};

mod pack;

pub use pack::{Pack, PackSources, Renderer};

/// Offsets of the four regions that share an edge with this one.
const NEIGHBOURS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const LAST_CHUNK: usize = CHUNKS_PER_SIDE - 1;

/// What one region wrote, stored as `regions/r.X.Z.json` in its dimension folder. Paths are relative to that folder.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RegionManifest {
	pub x: i32,
	pub z: i32,
	pub tile: String,
	pub sections: Vec<SectionEntry>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SectionEntry {
	pub x: i32,
	pub y: i32,
	pub z: i32,
	pub file: String,
	pub quads: usize,
}

/// Writes the tile, the section meshes and the manifest of the region at `path` under `dimension_dir`.
///
/// Culling needs the blocks along each shared edge, so the chunks on the facing edges of the neighbouring regions are
/// loaded for lookups but not drawn. A missing neighbour leaves its faces visible, the same as an unknown block would.
pub fn render_region(
	path: &Path,
	dimension_dir: &Path,
	defaults: &BlockDefaults,
	colours: &mut BlockColours<'_>,
	mesher: &mut MeshContext<'_>,
) -> Result<RegionManifest> {
	let region = RegionFile::open(path)?;
	let (region_x, region_z) = region.region_coords();
	let mut chunks = load_chunks(&region)?;
	chunks.extend(neighbour_edges(path, region_x, region_z));
	let world = BlockWorld::new(chunks, defaults);
	mesher.bake_world(&world);

	let stem = format!("r.{region_x}.{region_z}");
	let tile = format!("tiles/{stem}.png");
	write(&dimension_dir.join(&tile), &encode_png(&render_tile(&world, region_x, region_z, colours))?)?;

	let sections_dir = dimension_dir.join("sections").join(&stem);
	if sections_dir.exists() {
		fs::remove_dir_all(&sections_dir)?;
	}
	let mut sections = Vec::new();
	let owned = world.chunks().filter(|chunk| owns(chunk, region_x, region_z));
	for chunk in owned {
		for section in chunk.sections() {
			let Some(mesh) = mesher.mesh_section(&world, chunk.x, section.y(), chunk.z) else { continue };
			let quads: usize = mesh.layers.iter().map(Vec::len).sum();
			if quads == 0 {
				continue;
			}
			let name = format!("section_{}_{}_{}.obsm", chunk.x, section.y(), chunk.z);
			write(&sections_dir.join(&name), &format::encode(&mesh))?;
			sections.push(SectionEntry {
				x: chunk.x,
				y: section.y(),
				z: chunk.z,
				file: format!("sections/{stem}/{name}"),
				quads,
			});
		}
	}

	let manifest = RegionManifest {
		x: region_x,
		z: region_z,
		tile,
		sections,
	};
	let json = serde_json::to_vec(&manifest).map_err(std::io::Error::other)?;
	write(&dimension_dir.join("regions").join(format!("{stem}.json")), &json)?;
	Ok(manifest)
}

fn owns(chunk: &Chunk, region_x: i32, region_z: i32) -> bool {
	let size = CHUNKS_PER_SIDE as i32;
	chunk.x.div_euclid(size) == region_x && chunk.z.div_euclid(size) == region_z
}

fn load_chunks(region: &RegionFile) -> Result<Vec<Chunk>> {
	let mut chunks = Vec::new();
	for local_z in 0..CHUNKS_PER_SIDE {
		for local_x in 0..CHUNKS_PER_SIDE {
			if let Some(nbt) = region.chunk_nbt(local_x, local_z)? {
				chunks.push(Chunk::parse(&nbt)?);
			}
		}
	}
	Ok(chunks)
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
			let Ok(Some(nbt)) = neighbour.chunk_nbt(local_x, local_z) else { continue };
			if let Ok(chunk) = Chunk::parse(&nbt) {
				chunks.push(chunk);
			}
		}
	}
	chunks
}

fn write(path: &Path, bytes: &[u8]) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::write(path, bytes)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::atlas::{Atlas, block_textures};
	use crate::biome::BiomeRegistry;
	use crate::mesh::format;
	use crate::models::geometry::Direction;
	use crate::test_support::{chunk_nbt, region_bytes, stone_assets};

	fn solid_chunk(x: i32, z: i32) -> Vec<u8> {
		let blocks: Vec<((usize, usize, usize), &str)> = (0..16)
			.flat_map(|y| (0..16).flat_map(move |z| (0..16).map(move |x| ((x, y, z), "minecraft:stone"))))
			.collect();
		chunk_nbt(x, z, 3955, &blocks)
	}

	#[test]
	fn culls_faces_against_blocks_in_the_neighbouring_region() {
		let dir = tempfile::tempdir().unwrap();
		let west = dir.path().join("r.0.0.mca");
		let east = dir.path().join("r.1.0.mca");
		fs::write(&west, region_bytes(&[(31, 0, solid_chunk(31, 0))])).unwrap();
		fs::write(&east, region_bytes(&[(0, 0, solid_chunk(32, 0))])).unwrap();
		let assets = stone_assets();
		let biomes = BiomeRegistry::default();
		let atlas = Atlas::build(&assets, block_textures(&assets).iter().map(String::as_str));
		let mut mesher = MeshContext::new(&assets, &biomes, atlas);
		let mut colours = BlockColours::new(&assets, &biomes);
		let out = dir.path().join("out");
		let manifest = render_region(&west, &out, &BlockDefaults::default(), &mut colours, &mut mesher).unwrap();
		assert!(manifest.sections.iter().all(|section| section.x == 31), "only this region's chunks are drawn");
		let edge = manifest.sections.iter().find(|section| section.y == 0).expect("the edge section has visible faces");
		let mesh = format::decode(&fs::read(out.join(&edge.file)).unwrap()).unwrap();
		let east_faces = mesh.layers.iter().flatten().filter(|quad| quad.direction == Direction::East).count();
		assert_eq!(east_faces, 0, "faces against the neighbour's solid blocks are culled");
	}
}
