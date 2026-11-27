//! The blocks vanilla tints and the colour each takes: biome colours for grass, foliage, dry foliage and water, and fixed or
//! per-state colours for the rest. A block that is not listed stays untinted, even when its model carries a tint index.

use std::collections::BTreeMap;

use crate::models::geometry::{BlockTint, TintKind};

pub fn block_tint(name: &str, properties: &BTreeMap<String, String>) -> Option<BlockTint> {
	let id = name.strip_prefix("minecraft:").unwrap_or(name);
	match id {
		"fern" | "large_fern" | "short_grass" | "tall_grass" | "potted_fern" | "bush" | "pink_petals" | "wildflowers" | "grass_block"
		| "sugar_cane" => Some(BlockTint::Biome(TintKind::Grass)),
		"oak_leaves" | "jungle_leaves" | "acacia_leaves" | "dark_oak_leaves" | "mangrove_leaves" | "vine" => {
			Some(BlockTint::Biome(TintKind::Foliage))
		}
		"leaf_litter" => Some(BlockTint::Biome(TintKind::DryFoliage)),
		"water" | "water_cauldron" | "bubble_column" => Some(BlockTint::Biome(TintKind::Water)),
		"spruce_leaves" => Some(fixed(0x619961)),
		"birch_leaves" => Some(fixed(0x80a755)),
		"lily_pad" => Some(fixed(0x208030)),
		"attached_melon_stem" | "attached_pumpkin_stem" => Some(fixed(0xe0c71c)),
		"melon_stem" | "pumpkin_stem" => Some(BlockTint::Fixed(stem_colour(number(properties, "age")))),
		"redstone_wire" => Some(BlockTint::Fixed(redstone_colour(number(properties, "power")))),
		_ => None,
	}
}

fn fixed(colour: u32) -> BlockTint {
	BlockTint::Fixed([(colour >> 16) as u8, (colour >> 8) as u8, colour as u8])
}

fn number(properties: &BTreeMap<String, String>, name: &str) -> u32 {
	properties.get(name).and_then(|value| value.parse().ok()).unwrap_or(0)
}

/// Vanilla's stem colour: greener when young, yellower as the fruit ripens through eight ages.
fn stem_colour(age: u32) -> [u8; 3] {
	let age = age.min(7);
	[(age * 32) as u8, (255 - age * 8) as u8, (age * 4) as u8]
}

/// Vanilla's redstone colour for a signal strength, computed in single-precision floats as the game does.
fn redstone_colour(power: u32) -> [u8; 3] {
	let f = power.min(15) as f32 / 15.0;
	let red = f * 0.6 + if f > 0.0 { 0.4 } else { 0.3 };
	let green = (f * f * 0.7 - 0.5).clamp(0.0, 1.0);
	let blue = (f * f * 0.6 - 0.7).clamp(0.0, 1.0);
	[red, green, blue].map(|channel| (channel * 255.0).floor() as u8)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn props(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
		pairs.iter().map(|(name, value)| ((*name).to_owned(), (*value).to_owned())).collect()
	}

	#[test]
	fn grass_foliage_dry_foliage_and_water_take_biome_colours() {
		let none = BTreeMap::new();
		assert_eq!(block_tint("minecraft:tall_grass", &none), Some(BlockTint::Biome(TintKind::Grass)));
		assert_eq!(block_tint("minecraft:vine", &none), Some(BlockTint::Biome(TintKind::Foliage)));
		assert_eq!(block_tint("minecraft:leaf_litter", &none), Some(BlockTint::Biome(TintKind::DryFoliage)));
		assert_eq!(block_tint("minecraft:water_cauldron", &none), Some(BlockTint::Biome(TintKind::Water)));
	}

	#[test]
	fn leaves_and_plants_vanilla_leaves_untinted_stay_untinted() {
		let none = BTreeMap::new();
		for name in ["minecraft:pale_oak_leaves", "minecraft:cherry_leaves", "minecraft:azalea_leaves", "minecraft:bamboo", "minecraft:kelp"] {
			assert_eq!(block_tint(name, &none), None, "{name}");
		}
	}

	#[test]
	fn spruce_and_birch_leaves_take_fixed_colours() {
		let none = BTreeMap::new();
		assert_eq!(block_tint("minecraft:spruce_leaves", &none), Some(BlockTint::Fixed([0x61, 0x99, 0x61])));
		assert_eq!(block_tint("minecraft:birch_leaves", &none), Some(BlockTint::Fixed([0x80, 0xa7, 0x55])));
	}

	#[test]
	fn stems_and_redstone_follow_their_state() {
		assert_eq!(block_tint("minecraft:melon_stem", &props(&[("age", "7")])), Some(BlockTint::Fixed([224, 199, 28])));
		assert_eq!(block_tint("minecraft:redstone_wire", &props(&[("power", "0")])), Some(BlockTint::Fixed([76, 0, 0])));
		assert_eq!(block_tint("minecraft:redstone_wire", &props(&[("power", "15")])), Some(BlockTint::Fixed([255, 50, 0])));
	}
}
