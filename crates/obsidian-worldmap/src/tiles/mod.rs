pub mod detail;

use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};

use crate::biome::Biome;
use crate::chunk::Chunk;
use crate::colour::BlockColours;
use crate::error::Result;
use crate::world::BlockWorld;
use crate::region::CHUNKS_PER_SIDE;

const CHUNK_PIXELS: i32 = 16;
pub const REGION_PIXELS: u32 = CHUNKS_PER_SIDE as u32 * CHUNK_PIXELS as u32;
/// Brightness change per block of height difference against the northern neighbour, and the difference it stops at.
const SLOPE_STEP: f32 = 0.08;
const SLOPE_LIMIT: i32 = 3;

/// Top-down image of one region at one pixel per block. Columns without a generated chunk stay transparent.
pub fn render_tile(world: &BlockWorld, region_x: i32, region_z: i32, colours: &mut BlockColours) -> RgbaImage {
	let mut image = RgbaImage::new(REGION_PIXELS, REGION_PIXELS);
	let origin_x = region_x * CHUNKS_PER_SIDE as i32;
	let origin_z = region_z * CHUNKS_PER_SIDE as i32;
	for chunk in world.chunks() {
		draw_chunk(&mut image, world, chunk, origin_x, origin_z, colours);
	}
	image
}

fn draw_chunk(image: &mut RgbaImage, world: &BlockWorld, chunk: &Chunk, origin_x: i32, origin_z: i32, colours: &mut BlockColours) {
	let base_x = (chunk.x - origin_x) * CHUNK_PIXELS;
	let base_z = (chunk.z - origin_z) * CHUNK_PIXELS;
	let extent = 0..REGION_PIXELS as i32;
	let biomes = colours.biomes();
	for local_z in 0..CHUNK_PIXELS {
		for local_x in 0..CHUNK_PIXELS {
			let (px, pz) = (base_x + local_x, base_z + local_z);
			if !(extent.contains(&px) && extent.contains(&pz)) {
				continue;
			}
			let (block_x, block_z) = (local_x as usize, local_z as usize);
			if let Some((y, state)) = chunk.column_top(block_x, block_z) {
				let biome = chunk.biome_at(block_x, y, block_z).map_or(&Biome::PLAINS, |id| biomes.get(id));
				let world_x = chunk.x * CHUNK_PIXELS + local_x;
				let world_z = chunk.z * CHUNK_PIXELS + local_z;
				let factor = shade_factor(y) * slope_factor(y, top_height(world, world_x, world_z - 1));
				let [r, g, b] = shade(colours.colour(state, biome, world_x, world_z), factor);
				image.put_pixel(px as u32, pz as u32, Rgba([r, g, b, 255]));
			}
		}
	}
}

/// Darkens lower blocks so terrain reads as relief.
pub(crate) fn shade_factor(world_y: i32) -> f32 {
	let height = ((world_y + 64) as f32 / 384.0).clamp(0.0, 1.0);
	0.55 + 0.45 * height
}

/// Brightens a column that rises above its northern neighbour and darkens one that sits below it, as the in-game map
/// does, so slopes and cliffs read from above. Columns with no known neighbour are left as they are.
pub(crate) fn slope_factor(y: i32, north: Option<i32>) -> f32 {
	north.map_or(1.0, |north| 1.0 + (y - north).clamp(-SLOPE_LIMIT, SLOPE_LIMIT) as f32 * SLOPE_STEP)
}

/// Height of the highest block in the column at absolute block `x`, `z`, when its chunk is loaded.
pub(crate) fn top_height(world: &BlockWorld, x: i32, z: i32) -> Option<i32> {
	let chunk = world.chunk(x.div_euclid(CHUNK_PIXELS), z.div_euclid(CHUNK_PIXELS))?;
	chunk
		.column_top(x.rem_euclid(CHUNK_PIXELS) as usize, z.rem_euclid(CHUNK_PIXELS) as usize)
		.map(|(y, _)| y)
}

fn shade(colour: [u8; 3], factor: f32) -> [u8; 3] {
	colour.map(|channel| (f32::from(channel) * factor).min(255.0) as u8)
}

pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
	let mut png = Vec::new();
	image.write_to(&mut Cursor::new(&mut png), ImageFormat::Png)?;
	Ok(png)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::AssetStore;
	use crate::biome::BiomeRegistry;
	use crate::chunk::BlockDefaults;
	use crate::colour::fallback;
	use crate::test_support::chunk_nbt;

	fn world_with(blocks: &[((usize, usize, usize), &str)]) -> BlockWorld {
		BlockWorld::new([Chunk::parse(&chunk_nbt(0, 0, 3955, blocks)).unwrap()], &BlockDefaults::default())
	}

	#[test]
	fn draws_the_top_block_with_height_shading() {
		let world = world_with(&[((0, 0, 0), "minecraft:stone"), ((0, 1, 0), "minecraft:grass_block")]);
		let assets = AssetStore::default();
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let image = render_tile(&world, 0, 0, &mut colours);
		let [r, g, b] = shade(fallback("minecraft:grass_block"), shade_factor(1));
		assert_eq!(image.get_pixel(0, 0), &Rgba([r, g, b, 255]));
		assert_eq!(image.get_pixel(1, 0)[3], 0);
	}

	#[test]
	fn columns_above_their_northern_neighbour_are_brighter_and_those_below_darker() {
		let world = world_with(&[
			((0, 0, 0), "minecraft:stone"),
			((0, 2, 1), "minecraft:stone"),
			((0, 0, 2), "minecraft:stone"),
		]);
		let assets = AssetStore::default();
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let image = render_tile(&world, 0, 0, &mut colours);
		let flat = shade(fallback("minecraft:stone"), shade_factor(0))[0];
		assert_eq!(image.get_pixel(0, 0)[0], flat, "no neighbour to the north");
		assert!(image.get_pixel(0, 1)[0] > shade(fallback("minecraft:stone"), shade_factor(2))[0], "rises above the north");
		assert!(image.get_pixel(0, 2)[0] < flat, "sits below the north");
	}

	#[test]
	fn encodes_a_png_that_decodes_back_to_the_region_size() {
		let png = encode_png(&RgbaImage::new(REGION_PIXELS, REGION_PIXELS)).unwrap();
		let decoded = image::load_from_memory(&png).unwrap();
		assert_eq!((decoded.width(), decoded.height()), (REGION_PIXELS, REGION_PIXELS));
	}
}
