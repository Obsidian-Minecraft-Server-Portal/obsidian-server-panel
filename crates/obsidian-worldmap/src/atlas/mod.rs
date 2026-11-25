//! One texture atlas for the textures baked models use. Each texture takes a 16x16 cell. UVs stay in image
//! space (origin at the top left), so renderers must not flip them.

use std::collections::{BTreeMap, BTreeSet};

use image::imageops::{self, FilterType};
use image::{Rgba, RgbaImage};

use crate::assets::AssetStore;
use crate::models::geometry::{MISSING_TEXTURE, resolve_texture};
use crate::models::{Blockstate, resolve_model, split_id};

pub const CELL: u32 = 16;
const TRANSLUCENT_WORDS: [&str; 3] = ["glass", "ice", "water"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
	Opaque,
	Cutout,
	Translucent,
}

#[derive(Debug, Clone, Copy)]
struct Slot {
	column: u32,
	row: u32,
	layer: Layer,
}

pub struct Atlas {
	image: RgbaImage,
	slots: BTreeMap<String, Slot>,
}

impl Atlas {
	/// Packs `textures` and the missing-texture cell. A texture that cannot be read gets the missing cell.
	pub fn build<'a>(assets: &AssetStore, textures: impl IntoIterator<Item = &'a str>) -> Self {
		let mut ids: BTreeSet<&str> = textures.into_iter().collect();
		ids.insert(MISSING_TEXTURE);
		let count = ids.len() as u32;
		let columns = f64::from(count).sqrt().ceil() as u32;
		let rows = count.div_ceil(columns);
		let mut image = RgbaImage::new(columns * CELL, rows * CELL);
		let mut slots = BTreeMap::new();
		for (index, id) in ids.into_iter().enumerate() {
			let index = index as u32;
			let (column, row) = (index % columns, index / columns);
			let (pixels, layer) = load_texture(assets, id).unwrap_or_else(|| (missing_cell(), Layer::Opaque));
			imageops::replace(&mut image, &pixels, i64::from(column * CELL), i64::from(row * CELL));
			slots.insert(id.to_owned(), Slot { column, row, layer });
		}
		Self { image, slots }
	}

	pub fn image(&self) -> &RgbaImage {
		&self.image
	}

	pub fn layer(&self, texture: &str) -> Layer {
		self.slot(texture).layer
	}

	/// Maps a texture coordinate in 0..16 texture pixels to atlas coordinates in 0..1.
	pub fn uv(&self, texture: &str, uv: [f32; 2]) -> [f32; 2] {
		let slot = self.slot(texture);
		let (width, height) = self.image.dimensions();
		[
			((slot.column * CELL) as f32 + uv[0].clamp(0.0, CELL as f32)) / width as f32,
			((slot.row * CELL) as f32 + uv[1].clamp(0.0, CELL as f32)) / height as f32,
		]
	}

	fn slot(&self, texture: &str) -> Slot {
		self.slots.get(texture).copied().unwrap_or_else(|| self.slots[MISSING_TEXTURE])
	}
}

/// Every texture a blockstate in `assets` can draw. An atlas built from this set serves every region of the same assets,
/// so UVs stay valid as regions are re-rendered one at a time.
pub fn block_textures(assets: &AssetStore) -> BTreeSet<String> {
	let mut textures = BTreeSet::new();
	for (_, _, json) in assets.entries_under("blockstates/") {
		let Ok(blockstate) = Blockstate::parse(json) else { continue };
		for model in blockstate.model_ids() {
			let Ok(resolved) = resolve_model(assets, model) else { continue };
			let faces = resolved.elements.iter().flat_map(|element| element.faces.values().map(|face| face.texture.as_str()));
			let references = resolved.textures.values().map(String::as_str).chain(faces);
			textures.extend(references.map(|reference| resolve_texture(reference, &resolved.textures)));
		}
	}
	textures
}

/// The top square of the texture, so animated frame stacks use their first frame.
fn load_texture(assets: &AssetStore, id: &str) -> Option<(RgbaImage, Layer)> {
	let (namespace, path) = split_id(id);
	let png = assets.get(namespace, &format!("textures/{path}.png"))?;
	let decoded = image::load_from_memory(png).ok()?.to_rgba8();
	let side = decoded.width().min(decoded.height());
	if side == 0 {
		return None;
	}
	let square = imageops::crop_imm(&decoded, 0, 0, side, side).to_image();
	let layer = classify(id, &square);
	let cell = if side == CELL {
		square
	} else {
		imageops::resize(&square, CELL, CELL, FilterType::Triangle)
	};
	Some((cell, layer))
}

/// Read from the original pixels, before any resizing blurs alpha values together.
fn classify(id: &str, pixels: &RgbaImage) -> Layer {
	if TRANSLUCENT_WORDS.iter().any(|word| id.contains(*word)) {
		return Layer::Translucent;
	}
	let (mut partial, mut empty) = (false, false);
	for pixel in pixels.pixels() {
		match pixel[3] {
			255 => {}
			0 => empty = true,
			_ => partial = true,
		}
	}
	match (partial, empty) {
		(true, _) => Layer::Translucent,
		(false, true) => Layer::Cutout,
		(false, false) => Layer::Opaque,
	}
}

fn missing_cell() -> RgbaImage {
	RgbaImage::from_fn(CELL, CELL, |x, y| {
		if (x / 8 + y / 8) % 2 == 0 {
			Rgba([255, 0, 255, 255])
		} else {
			Rgba([0, 0, 0, 255])
		}
	})
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use image::{ImageFormat, Rgba, RgbaImage};

	use super::*;

	fn png(pixels: &[[u8; 4]]) -> Vec<u8> {
		let mut image = RgbaImage::new(16, 16);
		for (index, pixel) in pixels.iter().cycle().take(256).enumerate() {
			image.put_pixel(index as u32 % 16, index as u32 / 16, Rgba(*pixel));
		}
		let mut bytes = Vec::new();
		image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).unwrap();
		bytes
	}

	fn assets() -> AssetStore {
		let mut assets = AssetStore::default();
		assets.insert("minecraft".into(), "textures/block/a.png".into(), png(&[[200, 0, 0, 255]]));
		assets.insert("minecraft".into(), "textures/block/b.png".into(), png(&[[0, 0, 0, 0], [0, 200, 0, 255]]));
		assets.insert("minecraft".into(), "textures/block/glass.png".into(), png(&[[255, 255, 255, 255]]));
		assets
	}

	#[test]
	fn collects_the_textures_of_every_blockstate_model() {
		let mut assets = assets();
		assets.insert("minecraft".into(), "blockstates/log.json".into(), br#"{"variants": {"axis=y": {"model": "minecraft:block/log_y"}}}"#.to_vec());
		assets.insert(
			"minecraft".into(),
			"models/block/log_y.json".into(),
			br##"{"textures": {"side": "block/a"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#side"}, "down": {"texture": "minecraft:block/b"}}}]}"##.to_vec(),
		);
		let textures: Vec<String> = block_textures(&assets).into_iter().collect();
		assert_eq!(textures, ["minecraft:block/a", "minecraft:block/b"]);
	}

	#[test]
	fn packs_textures_into_cells_with_the_missing_cell_last() {
		let atlas = Atlas::build(&assets(), ["minecraft:block/a", "minecraft:block/b"]);
		assert_eq!(atlas.image().dimensions(), (32, 32));
		assert_eq!(atlas.uv("minecraft:block/a", [0.0, 0.0]), [0.0, 0.0]);
		assert_eq!(atlas.uv("minecraft:block/b", [0.0, 0.0]), [0.5, 0.0]);
		assert_eq!(atlas.uv("minecraft:missing", [0.0, 0.0]), [0.0, 0.5]);
		assert_eq!(atlas.uv("minecraft:block/unknown", [16.0, 16.0]), [0.5, 1.0]);
	}

	#[test]
	fn classifies_layers_from_alpha_and_translucent_names() {
		let atlas = Atlas::build(&assets(), ["minecraft:block/a", "minecraft:block/b", "minecraft:block/glass"]);
		assert_eq!(atlas.layer("minecraft:block/a"), Layer::Opaque);
		assert_eq!(atlas.layer("minecraft:block/b"), Layer::Cutout);
		assert_eq!(atlas.layer("minecraft:block/glass"), Layer::Translucent);
	}
}
