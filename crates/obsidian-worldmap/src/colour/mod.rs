//! Block colours from the texture of each block's top face, tinted for grass and leaves.

mod texture;

use std::collections::HashMap;

use crate::assets::AssetStore;
use crate::chunk::BlockState;
use crate::models::{Blockstate, Face, ResolvedModel, resolve_model, split_id};

/// Vanilla's default temperature and humidity for plains, where the colormaps are sampled.
const PLAINS_TEMPERATURE: f32 = 0.8;
const PLAINS_HUMIDITY: f32 = 0.4;
/// Vanilla's plains water colour, used because fluids have no blockstate to read it from.
const PLAINS_WATER: [u8; 3] = [63, 118, 228];

#[derive(Clone, Copy)]
enum Tint {
	Grass,
	Foliage,
}

pub struct BlockColours<'a> {
	assets: &'a AssetStore,
	cache: HashMap<String, [u8; 3]>,
	grass: Option<[u8; 3]>,
	foliage: Option<[u8; 3]>,
}

impl<'a> BlockColours<'a> {
	pub fn new(assets: &'a AssetStore) -> Self {
		Self {
			assets,
			cache: HashMap::new(),
			grass: None,
			foliage: None,
		}
	}

	pub fn colour(&mut self, state: &BlockState) -> [u8; 3] {
		let key = cache_key(state);
		if let Some(colour) = self.cache.get(&key) {
			return *colour;
		}
		let colour = self.lookup(state).unwrap_or_else(|| fallback(&state.name));
		self.cache.insert(key, colour);
		colour
	}

	fn lookup(&mut self, state: &BlockState) -> Option<[u8; 3]> {
		if state.name == "minecraft:water" {
			let base = self.texture_colour("minecraft:block/water_still")?;
			return Some(multiply(base, PLAINS_WATER));
		}
		let (namespace, path) = split_id(&state.name);
		let json = self.assets.get(namespace, &format!("blockstates/{path}.json"))?;
		let variant = Blockstate::parse(json).ok()?.variants_for(&state.properties).into_iter().next()?;
		let model = resolve_model(self.assets, &variant.model).ok()?;
		let face = top_face(&model)?;
		let base = self.texture_colour(&face_texture(&model, face)?)?;
		match (face.tintindex.is_some(), tint_for(&state.name)) {
			(true, Some(Tint::Grass)) => Some(multiply(base, self.tint(Tint::Grass)?)),
			(true, Some(Tint::Foliage)) => Some(multiply(base, self.tint(Tint::Foliage)?)),
			_ => Some(base),
		}
	}

	fn texture_colour(&mut self, texture: &str) -> Option<[u8; 3]> {
		let key = format!("texture:{texture}");
		if let Some(colour) = self.cache.get(&key) {
			return Some(*colour);
		}
		let (namespace, path) = split_id(texture);
		let png = self.assets.get(namespace, &format!("textures/{path}.png"))?;
		let colour = texture::average(png).ok().flatten()?;
		self.cache.insert(key, colour);
		Some(colour)
	}

	fn tint(&mut self, kind: Tint) -> Option<[u8; 3]> {
		let (slot, name) = match kind {
			Tint::Grass => (self.grass, "grass"),
			Tint::Foliage => (self.foliage, "foliage"),
		};
		if let Some(colour) = slot {
			return Some(colour);
		}
		let png = self.assets.get("minecraft", &format!("textures/colormap/{name}.png"))?;
		let colour = texture::sample_colormap(png, PLAINS_TEMPERATURE, PLAINS_HUMIDITY).ok()?;
		match kind {
			Tint::Grass => self.grass = Some(colour),
			Tint::Foliage => self.foliage = Some(colour),
		}
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

/// Grass and leaves are the tinted blocks the rest of this crate handles; other tinted blocks stay untinted.
fn tint_for(name: &str) -> Option<Tint> {
	if name.ends_with("_leaves") {
		Some(Tint::Foliage)
	} else if ["grass_block", "grass", "fern", "tall_grass"].iter().any(|grass| name.ends_with(grass)) {
		Some(Tint::Grass)
	} else {
		None
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

	#[test]
	fn takes_the_colour_of_the_top_face_texture() {
		let mut assets = AssetStore::default();
		let add = |assets: &mut AssetStore, path: &str, bytes: Vec<u8>| assets.insert("minecraft".into(), path.into(), bytes);
		add(&mut assets, "blockstates/test_block.json", br#"{"variants": {"": {"model": "minecraft:block/test_cube"}}}"#.to_vec());
		add(
			&mut assets,
			"models/block/test_cube.json",
			br##"{"textures": {"all": "minecraft:block/test_top"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#all"}}}]}"##.to_vec(),
		);
		add(&mut assets, "textures/block/test_top.png", solid_png([200, 100, 50, 255]));
		let mut colours = BlockColours::new(&assets);
		let state = BlockState {
			name: "minecraft:test_block".into(),
			properties: BTreeMap::new(),
		};
		assert_eq!(colours.colour(&state), [200, 100, 50]);
	}

	#[test]
	fn falls_back_to_a_stable_colour_without_assets() {
		let assets = AssetStore::default();
		let mut colours = BlockColours::new(&assets);
		let state = BlockState {
			name: "minecraft:mystery".into(),
			properties: BTreeMap::new(),
		};
		assert_eq!(colours.colour(&state), fallback("minecraft:mystery"));
	}
}
