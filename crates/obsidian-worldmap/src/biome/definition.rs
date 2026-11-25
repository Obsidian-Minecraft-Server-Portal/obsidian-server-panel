//! A biome's climate and the colours it overrides, read from its worldgen JSON.

use serde::Deserialize;

use super::swamp;
use crate::error::{Result, WorldmapError};

const DEFAULT_WATER: [u8; 3] = [63, 118, 228];
const DEFAULT_CLIMATE: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrassModifier {
	None,
	DarkForest,
	Swamp,
}

impl GrassModifier {
	/// Applies the modifier to a grass colour at block column `x`, `z`.
	pub fn apply(self, colour: [u8; 3], x: i32, z: i32) -> [u8; 3] {
		match self {
			Self::None => colour,
			Self::DarkForest => dark_forest(colour),
			Self::Swamp => swamp::grass_colour(x, z),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Biome {
	pub temperature: f32,
	pub downfall: f32,
	pub grass: Option<[u8; 3]>,
	pub foliage: Option<[u8; 3]>,
	pub water: [u8; 3],
	pub grass_modifier: GrassModifier,
}

impl Biome {
	/// Stands in for a biome whose definition is unknown: 1.13 to 1.17 chunks store biomes as numbers that are not
	/// mapped yet, and a modded biome can be missing from the jars that were read.
	pub const PLAINS: Self = Self {
		temperature: 0.8,
		downfall: 0.4,
		grass: None,
		foliage: None,
		water: DEFAULT_WATER,
		grass_modifier: GrassModifier::None,
	};

	pub fn from_json(json: &[u8]) -> Result<Self> {
		let raw: RawBiome = serde_json::from_slice(json).map_err(|error| WorldmapError::Asset(error.to_string()))?;
		Ok(Self {
			temperature: raw.temperature,
			downfall: raw.downfall,
			grass: raw.effects.grass_color.and_then(Colour::rgb),
			foliage: raw.effects.foliage_color.and_then(Colour::rgb),
			water: raw.effects.water_color.and_then(Colour::rgb).unwrap_or(DEFAULT_WATER),
			grass_modifier: match raw.effects.grass_color_modifier.as_deref() {
				Some("dark_forest") => GrassModifier::DarkForest,
				Some("swamp") => GrassModifier::Swamp,
				_ => GrassModifier::None,
			},
		})
	}
}

/// Vanilla blends grass towards dark green in dark forests, one channel at a time with carries, as one packed integer.
fn dark_forest(colour: [u8; 3]) -> [u8; 3] {
	let packed = u32::from_be_bytes([0, colour[0], colour[1], colour[2]]);
	let [_, red, green, blue] = (((packed & 0xfefefe) + 0x28340a) >> 1).to_be_bytes();
	[red, green, blue]
}

#[derive(Deserialize)]
struct RawBiome {
	#[serde(default = "default_climate")]
	temperature: f32,
	#[serde(default = "default_climate")]
	downfall: f32,
	#[serde(default)]
	effects: RawEffects,
}

fn default_climate() -> f32 {
	DEFAULT_CLIMATE
}

#[derive(Deserialize, Default)]
struct RawEffects {
	grass_color: Option<Colour>,
	foliage_color: Option<Colour>,
	water_color: Option<Colour>,
	grass_color_modifier: Option<String>,
}

/// Colours are written as an integer, or as a `#rrggbb` string in newer versions.
#[derive(Deserialize)]
#[serde(untagged)]
enum Colour {
	Number(i64),
	Hex(String),
}

impl Colour {
	fn rgb(self) -> Option<[u8; 3]> {
		let value = match self {
			Self::Number(value) => u32::try_from(value).ok()?,
			Self::Hex(text) => u32::from_str_radix(text.trim_start_matches('#'), 16).ok()?,
		};
		Some([(value >> 16) as u8, (value >> 8) as u8, value as u8])
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reads_climate_and_colour_overrides() {
		let json = br##"{"temperature": 0.8, "downfall": 0.9, "effects": {"grass_color": 6975545, "foliage_color": "#aabbcc", "water_color": 4159204, "grass_color_modifier": "swamp"}}"##;
		let biome = Biome::from_json(json).unwrap();
		assert!((biome.temperature - 0.8).abs() < 1e-6);
		assert!((biome.downfall - 0.9).abs() < 1e-6);
		assert_eq!(biome.grass, Some([0x6a, 0x70, 0x39]));
		assert_eq!(biome.foliage, Some([0xaa, 0xbb, 0xcc]));
		assert_eq!(biome.water, [0x3f, 0x76, 0xe4]);
		assert_eq!(biome.grass_modifier, GrassModifier::Swamp);
	}

	#[test]
	fn fills_in_defaults_for_missing_fields() {
		let biome = Biome::from_json(b"{}").unwrap();
		assert_eq!((biome.temperature, biome.downfall), (0.5, 0.5));
		assert_eq!((biome.grass, biome.foliage), (None, None));
		assert_eq!(biome.water, [63, 118, 228]);
		assert_eq!(biome.grass_modifier, GrassModifier::None);
	}

	#[test]
	fn dark_forest_blend_matches_the_vanilla_formula() {
		assert_eq!(dark_forest([0x6a, 0x70, 0x39]), [0x49, 0x52, 0x21]);
		assert_eq!(dark_forest([0x8d, 0xb3, 0x60]), [0x5a, 0x73, 0x35]);
	}

	#[test]
	fn rejects_files_that_are_not_json() {
		assert!(Biome::from_json(b"not json").is_err());
	}
}
