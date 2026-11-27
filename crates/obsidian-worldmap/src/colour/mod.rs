//! Block colours from the texture of each block's top face, tinted by the biome the block is in, and the top textures
//! themselves for the textured detail tiles.

mod texture;
mod tints;

use std::collections::HashMap;

use image::{Rgba, RgbaImage};

use crate::assets::AssetStore;
use crate::biome::{Biome, BiomeRegistry};
use crate::chunk::BlockState;
use crate::models::geometry::{TintKind, tint_kind_for_block};
use crate::models::{Blockstate, Face, ResolvedModel, resolve_model, split_id};
use texture::Decoded;

pub use tints::Tints;

/// Pixels per side of a block's top texture in a sprite.
pub const SPRITE_SIZE: u32 = 8;

#[derive(Clone)]
struct Surface {
	sprite: RgbaImage,
	average: [u8; 3],
	tint: Option<TintKind>,
	/// A plant or other sprite with no top face of its own. The maps leave these out and show the ground beneath.
	decoration: bool,
}

/// Decoded textures and block surfaces. Carrying one cache from call to call over the same assets decodes each texture once.
#[derive(Default)]
pub struct SurfaceCache {
	surfaces: HashMap<BlockState, Surface>,
	textures: HashMap<String, Option<Decoded>>,
}

impl SurfaceCache {
	fn surface(&mut self, assets: &AssetStore, state: &BlockState) -> &Surface {
		if !self.surfaces.contains_key(state) {
			let surface = self.lookup(assets, state).unwrap_or_else(|| flat(fallback(&state.name)));
			self.surfaces.insert(state.clone(), surface);
		}
		&self.surfaces[state]
	}

	fn lookup(&mut self, assets: &AssetStore, state: &BlockState) -> Option<Surface> {
		if state.name == "minecraft:water" {
			let decoded = self.texture(assets, "minecraft:block/water_still")?;
			return Some(surface_of(decoded, Some(TintKind::Water)));
		}
		let (namespace, path) = split_id(&state.name);
		let json = assets.get(namespace, &format!("blockstates/{path}.json"))?;
		let variant = Blockstate::parse(json).ok()?.variants_for(&state.properties).into_iter().next()?;
		let model = resolve_model(assets, &variant.model).ok()?;
		let Some(face) = top_face(&model) else {
			return Some(Surface {
				decoration: true,
				..flat(fallback(&state.name))
			});
		};
		let decoded = self.texture(assets, &face_texture(&model, face)?)?;
		let tint = match (face.tintindex.is_some(), tint_kind_for_block(&state.name)) {
			(true, Some(kind)) => Some(kind),
			_ => None,
		};
		Some(surface_of(decoded, tint))
	}

	fn texture(&mut self, assets: &AssetStore, texture: &str) -> Option<Decoded> {
		if let Some(decoded) = self.textures.get(texture) {
			return decoded.clone();
		}
		let (namespace, path) = split_id(texture);
		let decoded = assets
			.get(namespace, &format!("textures/{path}.png"))
			.and_then(|png| texture::decode(png, SPRITE_SIZE).ok().flatten());
		self.textures.insert(texture.to_owned(), decoded.clone());
		decoded
	}
}

pub struct BlockColours<'a> {
	assets: &'a AssetStore,
	biomes: &'a BiomeRegistry,
	tints: Tints,
	cache: SurfaceCache,
}

impl<'a> BlockColours<'a> {
	pub fn new(assets: &'a AssetStore, biomes: &'a BiomeRegistry) -> Self {
		Self::with_cache(assets, biomes, SurfaceCache::default())
	}

	/// Continues with the surfaces an earlier run over the same assets decoded.
	pub fn with_cache(assets: &'a AssetStore, biomes: &'a BiomeRegistry, cache: SurfaceCache) -> Self {
		Self {
			assets,
			biomes,
			tints: Tints::new(assets),
			cache,
		}
	}

	pub fn into_cache(self) -> SurfaceCache {
		self.cache
	}

	pub fn biomes(&self) -> &'a BiomeRegistry {
		self.biomes
	}

	/// The colour of a block's top face at block column `x`, `z` inside `biome`.
	pub fn colour(&mut self, state: &BlockState, biome: &Biome, x: i32, z: i32) -> [u8; 3] {
		let surface = self.cache.surface(self.assets, state);
		let average = surface.average;
		match surface.tint {
			Some(kind) => multiply(average, self.tints.colour(kind, biome, x, z)),
			None => average,
		}
	}

	/// Whether the block is a plant or other decoration with no top face, which the top-down tiles skip.
	pub fn is_decoration(&mut self, state: &BlockState) -> bool {
		self.cache.surface(self.assets, state).decoration
	}

	/// The top face's texture at sprite size, and the colour its texels are multiplied by inside `biome`.
	pub fn texels(&mut self, state: &BlockState, biome: &Biome, x: i32, z: i32) -> (&RgbaImage, [u8; 3]) {
		let surface = self.cache.surface(self.assets, state);
		let tint = surface.tint.map_or([255, 255, 255], |kind| self.tints.colour(kind, biome, x, z));
		(&surface.sprite, tint)
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

fn surface_of(decoded: Decoded, tint: Option<TintKind>) -> Surface {
	Surface {
		sprite: decoded.sprite,
		average: decoded.average,
		tint,
		decoration: false,
	}
}

fn flat(colour: [u8; 3]) -> Surface {
	Surface {
		sprite: RgbaImage::from_pixel(SPRITE_SIZE, SPRITE_SIZE, Rgba([colour[0], colour[1], colour[2], 255])),
		average: colour,
		tint: None,
		decoration: false,
	}
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

	fn grass_state() -> BlockState {
		BlockState {
			name: "minecraft:grass_block".into(),
			properties: BTreeMap::new(),
		}
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
		let swamp = Biome {
			grass: Some([10, 20, 30]),
			..Biome::PLAINS
		};
		assert_eq!(colours.colour(&grass_state(), &swamp, 0, 0), [10, 20, 30]);
		assert_eq!(colours.colour(&grass_state(), &Biome::PLAINS, 0, 0), [145, 189, 89]);
	}

	#[test]
	fn texels_are_the_top_texture_at_sprite_size_with_the_biome_tint() {
		let mut assets = AssetStore::default();
		add(&mut assets, "blockstates/grass_block.json", br#"{"variants": {"": {"model": "minecraft:block/grass_block"}}}"#.to_vec());
		add(
			&mut assets,
			"models/block/grass_block.json",
			br##"{"textures": {"top": "minecraft:block/grass_top"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#top", "tintindex": 0}}}]}"##.to_vec(),
		);
		add(&mut assets, "textures/block/grass_top.png", solid_png([200, 200, 200, 255]));
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let swamp = Biome {
			grass: Some([10, 20, 30]),
			..Biome::PLAINS
		};
		let (sprite, tint) = colours.texels(&grass_state(), &swamp, 0, 0);
		assert_eq!(sprite.dimensions(), (SPRITE_SIZE, SPRITE_SIZE));
		assert_eq!(sprite.get_pixel(3, 3), &Rgba([200, 200, 200, 255]));
		assert_eq!(tint, [10, 20, 30]);
	}

	#[test]
	fn marks_cross_models_without_a_top_face_as_decorations() {
		let mut assets = AssetStore::default();
		add(&mut assets, "blockstates/test_flower.json", br#"{"variants": {"": {"model": "minecraft:block/test_cross"}}}"#.to_vec());
		add(
			&mut assets,
			"models/block/test_cross.json",
			br##"{"textures": {"cross": "minecraft:block/test_flower"}, "elements": [{"from": [0,0,8], "to": [16,16,8], "faces": {"north": {"texture": "#cross"}, "south": {"texture": "#cross"}}}]}"##.to_vec(),
		);
		add(&mut assets, "blockstates/test_cube.json", br#"{"variants": {"": {"model": "minecraft:block/test_cube"}}}"#.to_vec());
		add(
			&mut assets,
			"models/block/test_cube.json",
			br##"{"textures": {"all": "minecraft:block/test_top"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#all"}}}]}"##.to_vec(),
		);
		add(&mut assets, "textures/block/test_top.png", solid_png([200, 100, 50, 255]));
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let flower = BlockState {
			name: "minecraft:test_flower".into(),
			properties: BTreeMap::new(),
		};
		let cube = BlockState {
			name: "minecraft:test_cube".into(),
			properties: BTreeMap::new(),
		};
		assert!(colours.is_decoration(&flower));
		assert!(!colours.is_decoration(&cube));
		assert!(!colours.is_decoration(&grass_state()), "without assets a block is a surface, not a decoration");
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
