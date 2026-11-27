//! Biome tints: the vanilla grass and foliage colormaps, overridden by each biome's own colours and grass modifier.

use crate::assets::AssetStore;
use crate::biome::Biome;
use crate::colour::texture::Colormap;
use crate::models::geometry::TintKind;

/// Used when a colormap is missing, so tinted blocks still get vanilla's plains colour.
const DEFAULT_GRASS: [u8; 3] = [145, 189, 89];
const DEFAULT_FOLIAGE: [u8; 3] = [119, 171, 47];
const DEFAULT_DRY_FOLIAGE: [u8; 3] = [92, 60, 50];

pub struct Tints {
	grass: Option<Colormap>,
	foliage: Option<Colormap>,
	dry_foliage: Option<Colormap>,
}

impl Tints {
	pub fn new(assets: &AssetStore) -> Self {
		Self {
			grass: load_colormap(assets, "grass"),
			foliage: load_colormap(assets, "foliage"),
			dry_foliage: load_colormap(assets, "dry_foliage"),
		}
	}

	/// The tint for a block of `kind` in `biome`, at block column `x`, `z`.
	pub fn colour(&self, kind: TintKind, biome: &Biome, x: i32, z: i32) -> [u8; 3] {
		match kind {
			TintKind::Grass => biome.grass_modifier.apply(self.base_colour(kind, biome), x, z),
			TintKind::Water | TintKind::Foliage | TintKind::DryFoliage => self.base_colour(kind, biome),
		}
	}

	/// The tint before any modifier that depends on the block's position.
	pub fn base_colour(&self, kind: TintKind, biome: &Biome) -> [u8; 3] {
		match kind {
			TintKind::Water => biome.water,
			TintKind::Grass => biome.grass.unwrap_or_else(|| sample(self.grass.as_ref(), biome, DEFAULT_GRASS)),
			TintKind::Foliage => biome.foliage.unwrap_or_else(|| sample(self.foliage.as_ref(), biome, DEFAULT_FOLIAGE)),
			TintKind::DryFoliage => biome.dry_foliage.unwrap_or_else(|| sample(self.dry_foliage.as_ref(), biome, DEFAULT_DRY_FOLIAGE)),
		}
	}
}

fn sample(colormap: Option<&Colormap>, biome: &Biome, default: [u8; 3]) -> [u8; 3] {
	colormap.map_or(default, |colormap| colormap.sample(biome.temperature, biome.downfall))
}

fn load_colormap(assets: &AssetStore, name: &str) -> Option<Colormap> {
	let png = assets.get("minecraft", &format!("textures/colormap/{name}.png"))?;
	Colormap::decode(png).ok()
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use image::{ImageFormat, Rgba, RgbaImage};

	use super::*;
	use crate::biome::GrassModifier;

	fn assets_with_grass_colormap() -> AssetStore {
		let mut colormap = RgbaImage::new(2, 2);
		colormap.put_pixel(0, 0, Rgba([10, 20, 30, 255]));
		let mut png = Vec::new();
		colormap.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();
		let mut assets = AssetStore::default();
		assets.insert("minecraft".into(), "textures/colormap/grass.png".into(), png);
		assets
	}

	#[test]
	fn samples_the_colormap_at_the_biome_climate_and_falls_back_to_vanilla() {
		let assets = assets_with_grass_colormap();
		let tints = Tints::new(&assets);
		assert_eq!(tints.colour(TintKind::Grass, &Biome::PLAINS, 0, 0), [10, 20, 30]);
		assert_eq!(tints.colour(TintKind::Foliage, &Biome::PLAINS, 0, 0), DEFAULT_FOLIAGE);
		assert_eq!(tints.colour(TintKind::Water, &Biome::PLAINS, 0, 0), [63, 118, 228]);
	}

	#[test]
	fn biome_colours_override_the_colormaps() {
		let biome = Biome {
			grass: Some([1, 2, 3]),
			foliage: Some([4, 5, 6]),
			water: [7, 8, 9],
			..Biome::PLAINS
		};
		let tints = Tints::new(&AssetStore::default());
		assert_eq!(tints.colour(TintKind::Grass, &biome, 0, 0), [1, 2, 3]);
		assert_eq!(tints.colour(TintKind::Foliage, &biome, 0, 0), [4, 5, 6]);
		assert_eq!(tints.colour(TintKind::Water, &biome, 0, 0), [7, 8, 9]);
	}

	#[test]
	fn dry_foliage_samples_its_colormap_and_takes_the_biome_override() {
		let mut colormap = RgbaImage::new(2, 2);
		colormap.put_pixel(0, 0, Rgba([40, 50, 60, 255]));
		let mut png = Vec::new();
		colormap.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();
		let mut assets = AssetStore::default();
		assets.insert("minecraft".into(), "textures/colormap/dry_foliage.png".into(), png);
		let tints = Tints::new(&assets);
		assert_eq!(tints.colour(TintKind::DryFoliage, &Biome::PLAINS, 0, 0), [40, 50, 60]);
		let biome = Biome {
			dry_foliage: Some([1, 2, 3]),
			..Biome::PLAINS
		};
		assert_eq!(tints.colour(TintKind::DryFoliage, &biome, 0, 0), [1, 2, 3]);
		assert_eq!(Tints::new(&AssetStore::default()).colour(TintKind::DryFoliage, &Biome::PLAINS, 0, 0), DEFAULT_DRY_FOLIAGE);
	}

	#[test]
	fn dark_forest_modifier_applies_to_the_grass_tint() {
		let biome = Biome {
			grass: Some([0x6a, 0x70, 0x39]),
			grass_modifier: GrassModifier::DarkForest,
			..Biome::PLAINS
		};
		let tints = Tints::new(&AssetStore::default());
		assert_eq!(tints.colour(TintKind::Grass, &biome, 0, 0), [0x49, 0x52, 0x21]);
	}
}
