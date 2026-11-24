//! Biome tints taken from the vanilla grass and foliage colormaps, sampled at plains conditions.

use crate::assets::AssetStore;
use crate::colour::texture;
use crate::models::geometry::TintKind;

/// Vanilla's default temperature and humidity for plains, where the colormaps are sampled.
const PLAINS_TEMPERATURE: f32 = 0.8;
const PLAINS_HUMIDITY: f32 = 0.4;
/// Vanilla's plains water colour. Fluids have no colormap to sample it from.
const PLAINS_WATER: [u8; 3] = [63, 118, 228];
/// Used when the colormap is missing, so tinted blocks still get vanilla's plains colour.
const DEFAULT_GRASS: [u8; 3] = [145, 189, 89];
const DEFAULT_FOLIAGE: [u8; 3] = [119, 171, 47];

pub struct Tints<'a> {
	assets: &'a AssetStore,
	grass: Option<[u8; 3]>,
	foliage: Option<[u8; 3]>,
}

impl<'a> Tints<'a> {
	pub fn new(assets: &'a AssetStore) -> Self {
		Self {
			assets,
			grass: None,
			foliage: None,
		}
	}

	pub fn colour(&mut self, kind: TintKind) -> [u8; 3] {
		let (cached, name, default) = match kind {
			TintKind::Water => return PLAINS_WATER,
			TintKind::Grass => (self.grass, "grass", DEFAULT_GRASS),
			TintKind::Foliage => (self.foliage, "foliage", DEFAULT_FOLIAGE),
		};
		if let Some(colour) = cached {
			return colour;
		}
		let colour = self.sample(name).unwrap_or(default);
		match kind {
			TintKind::Grass => self.grass = Some(colour),
			TintKind::Foliage => self.foliage = Some(colour),
			TintKind::Water => {}
		}
		colour
	}

	fn sample(&self, name: &str) -> Option<[u8; 3]> {
		let png = self.assets.get("minecraft", &format!("textures/colormap/{name}.png"))?;
		texture::sample_colormap(png, PLAINS_TEMPERATURE, PLAINS_HUMIDITY).ok()
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use image::{ImageFormat, Rgba, RgbaImage};

	use super::*;

	#[test]
	fn samples_the_colormap_at_plains_conditions_and_falls_back_to_vanilla() {
		let mut colormap = RgbaImage::new(2, 2);
		colormap.put_pixel(0, 0, Rgba([10, 20, 30, 255]));
		let mut png = Vec::new();
		colormap.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();

		let mut assets = AssetStore::default();
		assets.insert("minecraft".into(), "textures/colormap/grass.png".into(), png);
		let mut tints = Tints::new(&assets);
		assert_eq!(tints.colour(TintKind::Grass), [10, 20, 30]);
		assert_eq!(tints.colour(TintKind::Foliage), DEFAULT_FOLIAGE);
		assert_eq!(tints.colour(TintKind::Water), PLAINS_WATER);
	}
}
