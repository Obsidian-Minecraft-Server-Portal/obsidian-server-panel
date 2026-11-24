//! Section meshes: the faces of one 16x16x16 section that can be seen, grouped by render layer.

pub mod format;
mod world;

pub use world::BlockWorld;

use std::collections::{BTreeMap, HashMap};

use crate::assets::AssetStore;
use crate::atlas::{Atlas, Layer};
use crate::chunk::{BlockState, is_air};
use crate::colour::Tints;
use crate::models::geometry::{Direction, Quad};
use crate::models::bake_state;

/// Positions are stored in 1/256 of a block, relative to the section origin, as i16.
const POSITION_SCALE: f32 = 256.0;

#[derive(Debug, Clone, PartialEq)]
pub struct MeshQuad {
	pub corners: [[i16; 3]; 4],
	pub uvs: [[u16; 2]; 4],
	pub direction: Direction,
	pub tint: [u8; 3],
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
	atlas: Atlas,
	blocks: HashMap<String, Baked>,
	tints: Tints<'a>,
}

impl<'a> MeshContext<'a> {
	/// Bakes every block state in the world and packs their textures. Build once per region, then mesh sections.
	pub fn new(assets: &'a AssetStore, world: &BlockWorld) -> Self {
		let mut states: BTreeMap<String, BlockState> = BTreeMap::new();
		for chunk in world.chunks() {
			for section in chunk.sections() {
				for state in section.palette().iter().filter(|state| !is_air(&state.name)) {
					states.entry(block_key(state)).or_insert_with(|| state.clone());
				}
			}
		}
		let baked: Vec<(String, Vec<Quad>)> = states
			.iter()
			.map(|(key, state)| (key.clone(), bake_state(assets, state)))
			.collect();
		let atlas = Atlas::build(
			assets,
			baked.iter().flat_map(|(_, quads)| quads.iter().map(|quad| quad.texture.as_str())),
		);
		let blocks = baked
			.into_iter()
			.map(|(key, quads)| {
				let full = full_faces(&quads, &atlas);
				(key, Baked { quads, full })
			})
			.collect();
		Self {
			atlas,
			blocks,
			tints: Tints::new(assets),
		}
	}

	pub fn atlas(&self) -> &Atlas {
		&self.atlas
	}

	/// The section's visible faces. `None` when the chunk or section is not loaded.
	pub fn mesh_section(&mut self, world: &BlockWorld, chunk_x: i32, section_y: i32, chunk_z: i32) -> Option<SectionMesh> {
		let section = world.chunk(chunk_x, chunk_z)?.section(section_y)?;
		let mut layers: [Vec<MeshQuad>; 3] = Default::default();
		for local_y in 0..16 {
			for local_z in 0..16 {
				for local_x in 0..16 {
					let state = section.state_at(local_x, local_y, local_z);
					if is_air(&state.name) {
						continue;
					}
					let Some(baked) = self.blocks.get(&block_key(state)) else { continue };
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
							tint: quad.tint.map_or([255, 255, 255], |kind| self.tints.colour(kind)),
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

fn block_key(state: &BlockState) -> String {
	let properties: Vec<String> = state.properties.iter().map(|(name, value)| format!("{name}={value}")).collect();
	format!("{}[{}]", state.name, properties.join(","))
}

fn full_faces(quads: &[Quad], atlas: &Atlas) -> [bool; 6] {
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

fn is_culled(world: &BlockWorld, blocks: &HashMap<String, Baked>, state: &BlockState, quad: &Quad, at: [i32; 3]) -> bool {
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
		.get(&block_key(neighbour))
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
	use crate::chunk::{BlockDefaults, Chunk};
	use crate::test_support::chunk_nbt;

	fn stone_assets() -> AssetStore {
		let faces = r##"{"down": {"texture": "#all", "cullface": "down"}, "up": {"texture": "#all", "cullface": "up"}, "north": {"texture": "#all", "cullface": "north"}, "south": {"texture": "#all", "cullface": "south"}, "west": {"texture": "#all", "cullface": "west"}, "east": {"texture": "#all", "cullface": "east"}}"##;
		let model = format!(r##"{{"textures": {{"all": "minecraft:block/stone"}}, "elements": [{{"from": [0,0,0], "to": [16,16,16], "faces": {faces}}}]}}"##);
		let mut assets = AssetStore::default();
		assets.insert("minecraft".into(), "blockstates/stone.json".into(), br#"{"variants": {"": {"model": "minecraft:block/cube_all"}}}"#.to_vec());
		assets.insert("minecraft".into(), "models/block/cube_all.json".into(), model.into_bytes());
		let mut png = Vec::new();
		image::RgbaImage::from_pixel(16, 16, image::Rgba([120, 120, 120, 255]))
			.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
			.unwrap();
		assets.insert("minecraft".into(), "textures/block/stone.png".into(), png);
		assets
	}

	fn world_with(blocks: &[((usize, usize, usize), &str)]) -> BlockWorld {
		let chunk = Chunk::parse(&chunk_nbt(0, 0, 3955, blocks)).unwrap();
		BlockWorld::new([chunk], &BlockDefaults::default())
	}

	#[test]
	fn hides_the_face_shared_by_two_touching_opaque_blocks() {
		let assets = stone_assets();
		let world = world_with(&[((0, 0, 0), "minecraft:stone"), ((1, 0, 0), "minecraft:stone")]);
		let mut context = MeshContext::new(&assets, &world);
		let mesh = context.mesh_section(&world, 0, 0, 0).unwrap();
		assert_eq!(mesh.layers[Layer::Opaque as usize].len(), 10);
		assert!(mesh.layers[Layer::Cutout as usize].is_empty());
	}

	#[test]
	fn keeps_faces_next_to_unloaded_chunks_and_positions_inside_the_section() {
		let assets = stone_assets();
		let world = world_with(&[((15, 15, 15), "minecraft:stone")]);
		let mut context = MeshContext::new(&assets, &world);
		let mesh = context.mesh_section(&world, 0, 0, 0).unwrap();
		assert_eq!(mesh.layers[Layer::Opaque as usize].len(), 6);
		for quad in mesh.layers.iter().flatten() {
			for corner in quad.corners {
				assert!(corner.iter().all(|value| (0..=4096).contains(value)), "{corner:?}");
			}
		}
	}
}
