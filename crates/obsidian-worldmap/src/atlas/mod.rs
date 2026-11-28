//! One texture atlas for the textures baked models use. Block textures take a 16x16 cell. Entity textures take a 64x64 block, which
//! block UVs still address in units of 0 to 16. UVs stay in image space (origin at the top left), so renderers must not flip them.

use std::collections::{BTreeMap, BTreeSet};

use image::imageops::{self, FilterType};
use image::{Rgba, RgbaImage};

use crate::assets::AssetStore;
use crate::models::geometry::{MISSING_TEXTURE, resolve_texture};
use crate::models::{Blockstate, entity_textures, resolve_model, split_id};

pub const CELL: u32 = 16;
/// The side of an entity texture, which spans a block of four by four cells.
pub const ENTITY_SIZE: u32 = 64;
const TRANSLUCENT_WORDS: [&str; 3] = ["glass", "ice", "water"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
	Opaque,
	Cutout,
	Translucent,
}

#[derive(Debug, Clone, Copy)]
struct Slot {
	x: u32,
	y: u32,
	size: u32,
	layer: Layer,
}

pub struct Atlas {
	image: RgbaImage,
	slots: BTreeMap<String, Slot>,
}

impl Atlas {
	/// Packs `textures`, the `large` ones at their full entity size, and the missing-texture cell. A texture that cannot be read
	/// gets the missing cell.
	pub fn build<'a>(
		assets: &AssetStore,
		textures: impl IntoIterator<Item = &'a str>,
		translucent: &BTreeSet<String>,
		large: &BTreeSet<String>,
	) -> Self {
		let mut ids: BTreeSet<&str> = textures.into_iter().collect();
		ids.insert(MISSING_TEXTURE);
		let (entities, blocks): (Vec<&str>, Vec<&str>) = ids.into_iter().partition(|id| large.contains(*id));
		let cells_per_entity = (ENTITY_SIZE / CELL) * (ENTITY_SIZE / CELL);
		let cells = blocks.len() as u32 + entities.len() as u32 * cells_per_entity;
		let columns = (f64::from(cells).sqrt().ceil() as u32).next_multiple_of(ENTITY_SIZE / CELL);
		let entities_per_row = columns / (ENTITY_SIZE / CELL);
		let entities_top = (entities.len() as u32).div_ceil(entities_per_row) * ENTITY_SIZE;
		let blocks_top = entities_top;
		let block_rows = (blocks.len() as u32).div_ceil(columns);
		let mut atlas = Self {
			image: RgbaImage::new(columns * CELL, blocks_top + block_rows * CELL),
			slots: BTreeMap::new(),
		};
		for (index, id) in entities.iter().enumerate() {
			let index = index as u32;
			let origin = [(index % entities_per_row) * ENTITY_SIZE, (index / entities_per_row) * ENTITY_SIZE];
			atlas.place(assets, id, origin, ENTITY_SIZE, translucent.contains(*id));
		}
		for (index, id) in blocks.iter().enumerate() {
			let index = index as u32;
			let origin = [(index % columns) * CELL, blocks_top + (index / columns) * CELL];
			atlas.place(assets, id, origin, CELL, translucent.contains(*id));
		}
		atlas
	}

	fn place(&mut self, assets: &AssetStore, id: &str, [x, y]: [u32; 2], size: u32, forced_translucent: bool) {
		let (pixels, layer) = load_texture(assets, id, forced_translucent, size).unwrap_or_else(|| (missing_cell(size), Layer::Opaque));
		imageops::replace(&mut self.image, &pixels, i64::from(x), i64::from(y));
		self.slots.insert(id.to_owned(), Slot { x, y, size, layer });
	}

	pub fn image(&self) -> &RgbaImage {
		&self.image
	}

	pub fn layer(&self, texture: &str) -> Layer {
		self.slot(texture).layer
	}

	/// The top left texel of a texture's block in the atlas.
	pub fn origin(&self, texture: &str) -> [u32; 2] {
		let slot = self.slot(texture);
		[slot.x, slot.y]
	}

	/// The side of a texture's block in texels: 16 for a block texture, 64 for an entity texture.
	pub fn size(&self, texture: &str) -> u32 {
		self.slot(texture).size
	}

	/// Maps a texture coordinate in 0..16 units across the texture to atlas coordinates in 0..1.
	pub fn uv(&self, texture: &str, uv: [f32; 2]) -> [f32; 2] {
		let slot = self.slot(texture);
		let (width, height) = self.image.dimensions();
		let scale = slot.size as f32 / CELL as f32;
		[
			(slot.x as f32 + uv[0].clamp(0.0, CELL as f32) * scale) / width as f32,
			(slot.y as f32 + uv[1].clamp(0.0, CELL as f32) * scale) / height as f32,
		]
	}

	fn slot(&self, texture: &str) -> Slot {
		self.slots.get(texture).copied().unwrap_or_else(|| self.slots[MISSING_TEXTURE])
	}
}

/// Every texture a blockstate in `assets` can draw, with the entity textures the structure draws. An atlas built from this set
/// serves every region of the same assets, so UVs stay valid as regions are re-rendered one at a time.
#[derive(Default)]
pub struct BlockTextures {
	pub ids: BTreeSet<String>,
	/// Sprites that a model forces onto the translucent layer.
	pub translucent: BTreeSet<String>,
	/// Entity textures, which the atlas packs at their full size.
	pub large: BTreeSet<String>,
}

pub fn block_textures(assets: &AssetStore) -> BlockTextures {
	let mut textures = BlockTextures::default();
	for (_, _, json) in assets.entries_under("blockstates/") {
		let Ok(blockstate) = Blockstate::parse(json) else { continue };
		for model in blockstate.model_ids() {
			let Ok(resolved) = resolve_model(assets, model) else { continue };
			let faces = resolved.elements.iter().flat_map(|element| element.faces.values().map(|face| face.texture.as_str()));
			let references = resolved.textures.values().map(String::as_str).chain(faces);
			textures.ids.extend(references.map(|reference| resolve_texture(reference, &resolved.textures)));
			textures.translucent.extend(resolved.translucent);
		}
	}
	for texture in entity_textures() {
		textures.ids.insert(texture.clone());
		textures.large.insert(texture);
	}
	textures
}

/// The top square of the texture, so animated frame stacks use their first frame.
fn load_texture(assets: &AssetStore, id: &str, forced_translucent: bool, size: u32) -> Option<(RgbaImage, Layer)> {
	let (namespace, path) = split_id(id);
	let png = assets.get(namespace, &format!("textures/{path}.png"))?;
	let decoded = image::load_from_memory(png).ok()?.to_rgba8();
	let side = decoded.width().min(decoded.height());
	if side == 0 {
		return None;
	}
	let mut square = imageops::crop_imm(&decoded, 0, 0, side, side).to_image();
	let layer = classify(id, &square, forced_translucent);
	dilate_transparent(&mut square);
	let cell = if side == size {
		square
	} else {
		imageops::resize(&square, size, size, FilterType::Triangle)
	};
	Some((cell, layer))
}

/// Read from the original pixels, before any resizing blurs alpha values together.
fn classify(id: &str, pixels: &RgbaImage, forced_translucent: bool) -> Layer {
	if forced_translucent || TRANSLUCENT_WORDS.iter().any(|word| id.contains(*word)) {
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

/// Gives fully transparent texels the texture's mean colour, so the filtered mips of a cutout texture do not darken at its edges.
fn dilate_transparent(pixels: &mut RgbaImage) {
	let mut sum = [0_u64; 3];
	let mut count = 0_u64;
	for pixel in pixels.pixels().filter(|pixel| pixel[3] > 0) {
		for (channel, total) in sum.iter_mut().enumerate() {
			*total += u64::from(pixel[channel]);
		}
		count += 1;
	}
	if count == 0 {
		return;
	}
	let mean = sum.map(|total| (total / count) as u8);
	for pixel in pixels.pixels_mut().filter(|pixel| pixel[3] == 0) {
		for (channel, value) in mean.iter().enumerate() {
			pixel[channel] = *value;
		}
	}
}

fn missing_cell(size: u32) -> RgbaImage {
	let check = size / 2;
	RgbaImage::from_fn(size, size, |x, y| {
		if (x / check + y / check).is_multiple_of(2) {
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

	fn png(pixels: &[[u8; 4]], size: u32) -> Vec<u8> {
		let mut image = RgbaImage::new(size, size);
		for (index, pixel) in pixels.iter().cycle().take((size * size) as usize).enumerate() {
			image.put_pixel(index as u32 % size, index as u32 / size, Rgba(*pixel));
		}
		let mut bytes = Vec::new();
		image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).unwrap();
		bytes
	}

	fn assets() -> AssetStore {
		let mut assets = AssetStore::default();
		assets.insert("minecraft".into(), "textures/block/a.png".into(), png(&[[200, 0, 0, 255]], 16));
		assets.insert("minecraft".into(), "textures/block/b.png".into(), png(&[[0, 0, 0, 0], [0, 200, 0, 255]], 16));
		assets.insert("minecraft".into(), "textures/block/glass.png".into(), png(&[[255, 255, 255, 255]], 16));
		assets.insert("minecraft".into(), "textures/entity/chest/normal.png".into(), png(&[[90, 60, 30, 255]], 64));
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
		let textures = block_textures(&assets);
		let blocks: Vec<&str> = textures.ids.iter().map(String::as_str).filter(|id| !textures.large.contains(*id)).collect();
		assert_eq!(blocks, ["minecraft:block/a", "minecraft:block/b"]);
	}

	#[test]
	fn packs_textures_into_cells_with_the_missing_cell_last() {
		let atlas = Atlas::build(&assets(), ["minecraft:block/a", "minecraft:block/b"], &BTreeSet::new(), &BTreeSet::new());
		assert_eq!(atlas.image().dimensions(), (64, 16));
		assert_eq!(atlas.uv("minecraft:block/a", [0.0, 0.0]), [0.0, 0.0]);
		assert_eq!(atlas.uv("minecraft:block/b", [0.0, 0.0]), [0.25, 0.0]);
		assert_eq!(atlas.uv("minecraft:missing", [0.0, 0.0]), [0.5, 0.0]);
		assert_eq!(atlas.uv("minecraft:block/unknown", [16.0, 16.0]), [0.75, 1.0]);
	}

	#[test]
	fn entity_textures_pack_at_full_size_on_a_block_aligned_grid() {
		let large = BTreeSet::from(["minecraft:entity/chest/normal".to_owned()]);
		let atlas = Atlas::build(&assets(), ["minecraft:block/a", "minecraft:entity/chest/normal"], &BTreeSet::new(), &large);
		assert_eq!(atlas.size("minecraft:entity/chest/normal"), 64);
		assert_eq!(atlas.size("minecraft:block/a"), 16);
		assert_eq!(atlas.origin("minecraft:entity/chest/normal"), [0, 0]);
		assert!(atlas.origin("minecraft:block/a")[1] >= 64, "block cells sit below the entity blocks");
		let (width, _) = atlas.image().dimensions();
		let [left, _] = atlas.uv("minecraft:entity/chest/normal", [0.0, 0.0]);
		let [right, _] = atlas.uv("minecraft:entity/chest/normal", [16.0, 0.0]);
		assert_eq!(((right - left) * width as f32).round(), 64.0);
	}

	#[test]
	fn forced_translucency_overrides_the_alpha_rules() {
		let forced = BTreeSet::from(["minecraft:block/a".to_owned()]);
		let atlas = Atlas::build(&assets(), ["minecraft:block/a"], &forced, &BTreeSet::new());
		assert_eq!(atlas.layer("minecraft:block/a"), Layer::Translucent);
	}

	#[test]
	fn classifies_layers_from_alpha_and_translucent_names() {
		let atlas = Atlas::build(
			&assets(),
			["minecraft:block/a", "minecraft:block/b", "minecraft:block/glass"],
			&BTreeSet::new(),
			&BTreeSet::new(),
		);
		assert_eq!(atlas.layer("minecraft:block/a"), Layer::Opaque);
		assert_eq!(atlas.layer("minecraft:block/b"), Layer::Cutout);
		assert_eq!(atlas.layer("minecraft:block/glass"), Layer::Translucent);
	}
}
