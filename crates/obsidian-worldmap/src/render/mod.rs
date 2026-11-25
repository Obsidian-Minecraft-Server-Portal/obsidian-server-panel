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
pub fn render_region(
	path: &Path,
	dimension_dir: &Path,
	defaults: &BlockDefaults,
	colours: &mut BlockColours<'_>,
	mesher: &mut MeshContext<'_>,
) -> Result<RegionManifest> {
	let region = RegionFile::open(path)?;
	let (region_x, region_z) = region.region_coords();
	let world = BlockWorld::new(load_chunks(&region)?, defaults);
	mesher.bake_world(&world);

	let stem = format!("r.{region_x}.{region_z}");
	let tile = format!("tiles/{stem}.png");
	write(&dimension_dir.join(&tile), &encode_png(&render_tile(&world, region_x, region_z, colours))?)?;

	let sections_dir = dimension_dir.join("sections").join(&stem);
	if sections_dir.exists() {
		fs::remove_dir_all(&sections_dir)?;
	}
	let mut sections = Vec::new();
	for chunk in world.chunks() {
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

fn write(path: &Path, bytes: &[u8]) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::write(path, bytes)?;
	Ok(())
}
