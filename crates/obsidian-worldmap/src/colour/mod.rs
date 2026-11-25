//! Block colours from the texture of each block's top face, tinted by the biome the block is in.

mod texture;
mod tints;

use std::collections::HashMap;

use crate::assets::AssetStore;
use crate::biome::{Biome, BiomeRegistry};
use crate::chunk::BlockState;
use crate::models::geometry::{TintKind, tint_kind_for_block};
use crate::models::{Blockstate, Face, ResolvedModel, resolve_model, split_id};

pub use tints::Tints;

#[derive(Clone, Copy)]
enum Surface {
	Plain([u8; 3]),
	Tinted([u8; 3], TintKind),
}

pub struct BlockColours<'a> {
	assets: &'a AssetStore,
	biomes: &'a BiomeRegistry,
	tints: Tints,
	surfaces: HashMap<String, Surface>,
	textures: HashMap<String, [u8; 3]>,
}

impl<'a> BlockColours<'a> {
	pub fn new(assets: &'a AssetStore, biomes: &'a BiomeRegistry) -> Self {
		Self {
			assets,
			biomes,
			tints: Tints::new(assets),
			surfaces: HashMap::new(),
			textures: HashMap::new(),
		}
	}

	pub fn biomes(&self) -> &'a BiomeRegistry {
		self.biomes
	}

	/// The colour of a block's top face at block column `x`, `z` inside `biome`.
	pub fn colour(&mut self, state: &BlockState, biome: &Biome, x: i32, z: i32) -> [u8; 3] {
		let key = cache_key(state);
		let surface = if let Some(surface) = self.surfaces.get(&key).copied() {
			surface
		} else {
			let surface = self.lookup(state).unwrap_or(Surface::Plain(fallback(&state.name)));
			self.surfaces.insert(key, surface);
			surface
		};
		match surface {
			Surface::Plain(colour) => colour,
			Surface::Tinted(base, kind) => multiply(base, self.tints.colour(kind, biome, x, z)),
		}
	}

	fn lookup(&mut self, state: &BlockState) -> Option<Surface> {
		if state.name == "minecraft:water" {
			let base = self.texture_colour("minecraft:block/water_still")?;
			return Some(Surface::Tinted(base, TintKind::Water));
		}
		let (namespace, path) = split_id(&state.name);
		let json = self.assets.get(namespace, &format!("blockstates/{path}.json"))?;
		let variant = Blockstate::parse(json).ok()?.variants_for(&state.properties).into_iter().next()?;
		let model = resolve_model(self.assets, &variant.model).ok()?;
		let face = top_face(&model)?;
		let base = self.texture_colour(&face_texture(&model, face)?)?;
		Some(match (face.tintindex.is_some(), tint_kind_for_block(&state.name)) {
			(true, Some(kind)) => Surface::Tinted(base, kind),
			_ => Surface::Plain(base),
		})
	}

	fn texture_colour(&mut self, texture: &str) -> Option<[u8; 3]> {
		if let Some(colour) = self.textures.get(texture) {
			return Some(*colour);
		}
		let (namespace, path) = split_id(texture);
		let png = self.assets.get(namespace, &format!("textures/{path}.png"))?;
		let colour = texture::average(png).ok().flatten()?;
		self.textures.insert(texture.to_owned(), colour);
		Some(colour)
	}
}

/// Colour for a block with no assets loaded, so the map still has something to draw.
pub fn fallback(name: &str) -> [u8; 3] {
	match name {
		"minecraft:grass_block" => [121, 176, 66],
		"minecraft:water" => [63, 118, 228],
		"minecraft:sand" => [219, 211, 160],
		"minecraft:stone" => [125, 125, 125],
		"minecraft:snow_block" => [248, 250, 250],
		_ => hashed(name),
	}
}

fn cache_key(state: &BlockState) -> String {
	let properties: Vec<String> = state.properties.iter().map(|(name, value)| format!("{name}={value}")).collect();
	format!("{}[{}]", state.name, properties.join(","))
}

/// The element whose top edge is highest is the one showing on top of the column.
fn top_face(model: &ResolvedModel) -> Option<&Face> {
	model
		.elements
		.iter()
		.filter_map(|element| element.faces.get("up").map(|face| (element.to[1], face)))
		.max_by(|a, b| a.0.total_cmp(&b.0))
		.map(|(_, face)| face)
}

fn face_texture(model: &ResolvedModel, face: &Face) -> Option<String> {
	match face.texture.strip_prefix('#') {
		Some(variable) => model.textures.get(variable).cloned(),
		None => Some(face.texture.clone()),
	}
}

fn multiply(base: [u8; 3], tint: [u8; 3]) -> [u8; 3] {
	std::array::from_fn(|channel| (u16::from(base[channel]) * u16::from(tint[channel]) / 255) as u8)
}

fn hashed(name: &str) -> [u8; 3] {
	let hash = name
		.bytes()
		.fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3));
	[hash as u8 | 0x40, (hash >> 8) as u8 | 0x40, (hash >> 16) as u8 | 0x40]
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use std::io::Cursor;

	use image::{ImageFormat, Rgba, RgbaImage};

	use super::*;

	fn solid_png(rgba: [u8; 4]) -> Vec<u8> {
		let mut bytes = Vec::new();
		RgbaImage::from_pixel(2, 2, Rgba(rgba))
			.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
			.unwrap();
		bytes
	}

	fn add(assets: &mut AssetStore, path: &str, bytes: Vec<u8>) {
		assets.insert("minecraft".into(), path.into(), bytes);
	}

	#[test]
	fn takes_the_colour_of_the_top_face_texture() {
		let mut assets = AssetStore::default();
		add(&mut assets, "blockstates/test_block.json", br#"{"variants": {"": {"model": "minecraft:block/test_cube"}}}"#.to_vec());
		add(
			&mut assets,
			"models/block/test_cube.json",
			br##"{"textures": {"all": "minecraft:block/test_top"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#all"}}}]}"##.to_vec(),
		);
		add(&mut assets, "textures/block/test_top.png", solid_png([200, 100, 50, 255]));
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let state = BlockState {
			name: "minecraft:test_block".into(),
			properties: BTreeMap::new(),
		};
		assert_eq!(colours.colour(&state, &Biome::PLAINS, 0, 0), [200, 100, 50]);
	}

	#[test]
	fn tints_grass_tops_with_the_biome() {
		let mut assets = AssetStore::default();
		add(&mut assets, "blockstates/grass_block.json", br#"{"variants": {"": {"model": "minecraft:block/grass_block"}}}"#.to_vec());
		add(
			&mut assets,
			"models/block/grass_block.json",
			br##"{"textures": {"top": "minecraft:block/grass_top"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#top", "tintindex": 0}}}]}"##.to_vec(),
		);
		add(&mut assets, "textures/block/grass_top.png", solid_png([255, 255, 255, 255]));
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let state = BlockState {
			name: "minecraft:grass_block".into(),
			properties: BTreeMap::new(),
		};
		let swamp = Biome {
			grass: Some([10, 20, 30]),
			..Biome::PLAINS
		};
		assert_eq!(colours.colour(&state, &swamp, 0, 0), [10, 20, 30]);
		assert_eq!(colours.colour(&state, &Biome::PLAINS, 0, 0), [145, 189, 89]);
	}

	#[test]
	fn falls_back_to_a_stable_colour_without_assets() {
		let assets = AssetStore::default();
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let state = BlockState {
			name: "minecraft:mystery".into(),
			properties: BTreeMap::new(),
		};
		assert_eq!(colours.colour(&state, &Biome::PLAINS, 0, 0), fallback("minecraft:mystery"));
	}
}
