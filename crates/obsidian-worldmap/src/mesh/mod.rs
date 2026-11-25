//! Section meshes: the faces of one 16x16x16 section that can be seen, grouped by render layer.

pub mod format;
mod lighting;
mod world;

pub use world::BlockWorld;

use lighting::corner_brightness;

use std::collections::HashMap;

use crate::assets::AssetStore;
use crate::atlas::{Atlas, Layer};
use crate::biome::{Biome, BiomeRegistry};
use crate::chunk::{BlockState, is_air};
use crate::colour::Tints;
use crate::models::bake_state;
use crate::models::geometry::{Direction, Quad};

/// Positions are stored in 1/256 of a block, relative to the section origin, as i16.
const POSITION_SCALE: f32 = 256.0;

#[derive(Debug, Clone, PartialEq)]
pub struct MeshQuad {
	pub corners: [[i16; 3]; 4],
	pub uvs: [[u16; 2]; 4],
	pub direction: Direction,
	pub tint: [u8; 3],
	/// Brightness of each corner, 0 to 255, from ambient occlusion and light.
	pub light: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionMesh {
	/// Chunk x, section y and chunk z.
	pub section: [i32; 3],
	/// Indexed by [`Layer`]: opaque, cutout, translucent.
	pub layers: [Vec<MeshQuad>; 3],
}

struct Baked {
	quads: Vec<Quad>,
	/// Whether the block has a full-size opaque face on each side, indexed by [`Direction::index`].
	full: [bool; 6],
}

pub struct MeshContext<'a> {
	assets: &'a AssetStore,
	atlas: Atlas,
	blocks: HashMap<BlockState, Baked>,
	tints: Tints,
	biomes: &'a BiomeRegistry,
}

impl<'a> MeshContext<'a> {
	/// `atlas` should pack every texture the assets can draw (see [`crate::atlas::block_textures`]), so UVs stay valid
	/// while regions are rendered one at a time.
	pub fn new(assets: &'a AssetStore, biomes: &'a BiomeRegistry, atlas: Atlas) -> Self {
		Self {
			assets,
			atlas,
			blocks: HashMap::new(),
			tints: Tints::new(assets),
			biomes,
		}
	}

	pub fn atlas(&self) -> &Atlas {
		&self.atlas
	}

	/// Bakes the block states of `world` that earlier worlds did not need. Call before meshing the sections of `world`.
	pub fn bake_world(&mut self, world: &BlockWorld) {
		for chunk in world.chunks() {
			for section in chunk.sections() {
				for state in section.palette().iter().filter(|state| !is_air(&state.name)) {
					if !self.blocks.contains_key(state) {
						let quads = bake_state(self.assets, state);
						let full = full_faces(&quads, &self.atlas);
						self.blocks.insert(state.clone(), Baked { quads, full });
					}
				}
			}
		}
	}

	/// The section's visible faces. `None` when the chunk or section is not loaded.
	pub fn mesh_section(&self, world: &BlockWorld, chunk_x: i32, section_y: i32, chunk_z: i32) -> Option<SectionMesh> {
		let section = world.chunk(chunk_x, chunk_z)?.section(section_y)?;
		let biomes = self.biomes;
		let section_biomes: Vec<&Biome> = section.biome_palette().iter().map(|id| biomes.get(id)).collect();
		let mut layers: [Vec<MeshQuad>; 3] = Default::default();
		for local_y in 0..16 {
			for local_z in 0..16 {
				for local_x in 0..16 {
					let state = section.state_at(local_x, local_y, local_z);
					if is_air(&state.name) {
						continue;
					}
					let Some(baked) = self.blocks.get(state) else { continue };
					let biome = section
						.biome_index_at(local_x, local_y, local_z)
						.and_then(|index| section_biomes.get(index).copied())
						.unwrap_or(&Biome::PLAINS);
					let at = [
						chunk_x * 16 + local_x as i32,
						section_y * 16 + local_y as i32,
						chunk_z * 16 + local_z as i32,
					];
					for quad in &baked.quads {
						if is_culled(world, &self.blocks, state, quad, at) {
							continue;
						}
						let layer = self.atlas.layer(&quad.texture) as usize;
						layers[layer].push(MeshQuad {
							corners: quad.corners.map(|corner| corner_position(corner, [local_x, local_y, local_z])),
							uvs: quad.uvs.map(|uv| encode_uv(self.atlas.uv(&quad.texture, uv))),
							direction: quad.direction,
							tint: quad.tint.map_or([255, 255, 255], |kind| self.tints.colour(kind, biome, at[0], at[2])),
							light: corner_brightness(world, &self.blocks, quad, at),
						});
					}
				}
			}
		}
		Some(SectionMesh {
			section: [chunk_x, section_y, chunk_z],
			layers,
		})
	}
}

pub(crate) fn full_faces(quads: &[Quad], atlas: &Atlas) -> [bool; 6] {
	let mut full = [false; 6];
	for quad in quads {
		if atlas.layer(&quad.texture) == Layer::Opaque && covers_block_face(quad) {
			full[usize::from(quad.direction.index())] = true;
		}
	}
	full
}

/// A quad covers its block face when it lies on that face and spans the whole block in the other two axes.
fn covers_block_face(quad: &Quad) -> bool {
	let (axis, boundary) = match quad.direction {
		Direction::Down => (1, 0.0),
		Direction::Up => (1, 16.0),
		Direction::North => (2, 0.0),
		Direction::South => (2, 16.0),
		Direction::West => (0, 0.0),
		Direction::East => (0, 16.0),
	};
	if quad.corners.iter().any(|corner| (corner[axis] - boundary).abs() > 1e-3) {
		return false;
	}
	[0, 1, 2].into_iter().filter(|other| *other != axis).all(|other| {
		let low = quad.corners.iter().map(|corner| corner[other]).fold(f32::MAX, f32::min);
		let high = quad.corners.iter().map(|corner| corner[other]).fold(f32::MIN, f32::max);
		low.abs() < 1e-3 && (high - 16.0).abs() < 1e-3
	})
}

fn is_culled(world: &BlockWorld, blocks: &HashMap<BlockState, Baked>, state: &BlockState, quad: &Quad, at: [i32; 3]) -> bool {
	let Some(face) = quad.cullface else { return false };
	let [dx, dy, dz] = face.offset();
	let Some(neighbour) = world.state(at[0] + dx, at[1] + dy, at[2] + dz) else { return false };
	if is_air(&neighbour.name) {
		return false;
	}
	if neighbour.name == state.name && is_fluid(&state.name) {
		return true;
	}
	blocks
		.get(neighbour)
		.is_some_and(|baked| baked.full[usize::from(face.opposite().index())])
}

fn is_fluid(name: &str) -> bool {
	matches!(name, "minecraft:water" | "minecraft:lava")
}

fn corner_position(corner: [f32; 3], local: [usize; 3]) -> [i16; 3] {
	[0, 1, 2].map(|axis| ((local[axis] as f32 + corner[axis] / 16.0) * POSITION_SCALE).round() as i16)
}

fn encode_uv(uv: [f32; 2]) -> [u16; 2] {
	uv.map(|value| (value.clamp(0.0, 1.0) * 65535.0).round() as u16)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::atlas::block_textures;
	use crate::chunk::{BlockDefaults, Chunk};
	use crate::test_support::{chunk_nbt, chunk_nbt_with_biomes, stone_assets};

	fn grass_assets() -> AssetStore {
		let mut assets = AssetStore::default();
		assets.insert("minecraft".into(), "blockstates/grass_block.json".into(), br#"{"variants": {"": {"model": "minecraft:block/grass_block"}}}"#.to_vec());
		assets.insert(
			"minecraft".into(),
			"models/block/grass_block.json".into(),
			br##"{"textures": {"top": "minecraft:block/grass_top"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#top", "tintindex": 0, "cullface": "up"}}}]}"##.to_vec(),
		);
		let mut png = Vec::new();
		image::RgbaImage::from_pixel(16, 16, image::Rgba([255, 255, 255, 255]))
			.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
			.unwrap();
		assets.insert("minecraft".into(), "textures/block/grass_top.png".into(), png);
		assets
	}

	fn world_with(blocks: &[((usize, usize, usize), &str)]) -> BlockWorld {
		let chunk = Chunk::parse(&chunk_nbt(0, 0, 3955, blocks)).unwrap();
		BlockWorld::new([chunk], &BlockDefaults::default())
	}

	fn mesher<'a>(assets: &'a AssetStore, biomes: &'a BiomeRegistry, world: &BlockWorld) -> MeshContext<'a> {
		let atlas = Atlas::build(assets, block_textures(assets).iter().map(String::as_str));
		let mut context = MeshContext::new(assets, biomes, atlas);
		context.bake_world(world);
		context
	}

	#[test]
	fn hides_the_face_shared_by_two_touching_opaque_blocks() {
		let assets = stone_assets();
		let biomes = BiomeRegistry::default();
		let world = world_with(&[((0, 0, 0), "minecraft:stone"), ((1, 0, 0), "minecraft:stone")]);
		let context = mesher(&assets, &biomes, &world);
		let mesh = context.mesh_section(&world, 0, 0, 0).unwrap();
		assert_eq!(mesh.layers[Layer::Opaque as usize].len(), 10);
		assert!(mesh.layers[Layer::Cutout as usize].is_empty());
	}

	#[test]
	fn keeps_faces_next_to_unloaded_chunks_and_positions_inside_the_section() {
		let assets = stone_assets();
		let biomes = BiomeRegistry::default();
		let world = world_with(&[((15, 15, 15), "minecraft:stone")]);
		let context = mesher(&assets, &biomes, &world);
		let mesh = context.mesh_section(&world, 0, 0, 0).unwrap();
		assert_eq!(mesh.layers[Layer::Opaque as usize].len(), 6);
		for quad in mesh.layers.iter().flatten() {
			for corner in quad.corners {
				assert!(corner.iter().all(|value| (0..=4096).contains(value)), "{corner:?}");
			}
		}
	}

	#[test]
	fn corners_beside_solid_cubes_are_darker_than_open_corners() {
		let assets = stone_assets();
		let biomes = BiomeRegistry::default();
		let world = world_with(&[((0, 0, 0), "minecraft:stone"), ((1, 1, 0), "minecraft:stone")]);
		let context = mesher(&assets, &biomes, &world);
		let mesh = context.mesh_section(&world, 0, 0, 0).unwrap();
		let top = mesh.layers[Layer::Opaque as usize]
			.iter()
			.find(|quad| quad.direction == Direction::Up && quad.corners.iter().all(|corner| corner[1] == 256))
			.expect("the top of the lower block is visible");
		let corner_at = |x: i16, z: i16| top.corners.iter().position(|corner| corner[0] == x && corner[2] == z).unwrap();
		assert_eq!(top.light[corner_at(0, 0)], 255);
		assert!(top.light[corner_at(256, 0)] < top.light[corner_at(0, 0)]);
	}

	#[test]
	fn tints_grass_faces_with_the_biome_of_their_cell() {
		let assets = grass_assets();
		let chunk = Chunk::parse(&chunk_nbt_with_biomes(
			0,
			0,
			3955,
			&[((0, 0, 0), "minecraft:grass_block")],
			&[((0, 0, 0), "minecraft:swamp")],
		))
		.unwrap();
		let world = BlockWorld::new([chunk], &BlockDefaults::default());
		let mut biomes = BiomeRegistry::default();
		biomes.insert(
			"minecraft:swamp",
			Biome {
				grass: Some([10, 20, 30]),
				..Biome::PLAINS
			},
		);
		let context = mesher(&assets, &biomes, &world);
		let mesh = context.mesh_section(&world, 0, 0, 0).unwrap();
		let quads: Vec<&MeshQuad> = mesh.layers.iter().flatten().collect();
		assert_eq!(quads.len(), 1);
		assert_eq!(quads[0].tint, [10, 20, 30]);
	}
}
