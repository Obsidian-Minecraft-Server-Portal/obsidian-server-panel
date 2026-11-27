//! Textured top-down tiles for close zoom. A detail tile covers 32 by 32 blocks, two by two chunks, so drawing one reads
//! only the chunks it shows. Each block is drawn with its top texture at sprite size.

use image::{Rgba, RgbaImage};

use crate::biome::Biome;
use crate::chunk::{BlockDefaults, Chunk};
use crate::colour::{BlockColours, SPRITE_SIZE};
use crate::error::Result;
use crate::world::BlockWorld;
use crate::region::{CHUNKS_PER_SIDE, RegionFile};
use crate::tiles::{shade_factor, slope_factor, surface_top, top_height};

/// Detail tiles along one side of a region.
pub const TILES_PER_SIDE: usize = CHUNKS_PER_SIDE / 2;
/// Pixels along one side of a detail tile.
pub const TILE_PIXELS: u32 = 2 * 16 * SPRITE_SIZE;

const CHUNK_BLOCKS: usize = 16;
const CHUNKS_PER_TILE: usize = 2;

/// The detail tile at `tile_x` and `tile_z` within `region`, each below [`TILES_PER_SIDE`]. Columns without a block stay
/// transparent.
pub fn render_detail_tile(
	region: &RegionFile,
	tile_x: usize,
	tile_z: usize,
	defaults: &BlockDefaults,
	colours: &mut BlockColours,
) -> Result<RgbaImage> {
	let (region_x, region_z) = region.region_coords();
	let origin_x = region_x * CHUNKS_PER_SIDE as i32 + (tile_x * CHUNKS_PER_TILE) as i32;
	let origin_z = region_z * CHUNKS_PER_SIDE as i32 + (tile_z * CHUNKS_PER_TILE) as i32;
	let first_z = (tile_z * CHUNKS_PER_TILE).saturating_sub(1);
	let mut chunks = Vec::new();
	for local_z in first_z..(tile_z + 1) * CHUNKS_PER_TILE {
		for local_x in tile_x * CHUNKS_PER_TILE..(tile_x + 1) * CHUNKS_PER_TILE {
			if let Some(nbt) = region.chunk_nbt(local_x, local_z)? {
				chunks.push(Chunk::parse(&nbt)?);
			}
		}
	}
	let world = BlockWorld::new(chunks, defaults);
	let mut image = RgbaImage::new(TILE_PIXELS, TILE_PIXELS);
	for chunk in world.chunks() {
		draw_chunk(&mut image, &world, chunk, origin_x, origin_z, colours);
	}
	Ok(image)
}

fn draw_chunk(image: &mut RgbaImage, world: &BlockWorld, chunk: &Chunk, origin_x: i32, origin_z: i32, colours: &mut BlockColours) {
	let (offset_x, offset_z) = (chunk.x - origin_x, chunk.z - origin_z);
	let inside = 0..CHUNKS_PER_TILE as i32;
	if !inside.contains(&offset_x) || !inside.contains(&offset_z) {
		return;
	}
	let biomes = colours.biomes();
	for local_z in 0..CHUNK_BLOCKS {
		for local_x in 0..CHUNK_BLOCKS {
			let world_x = chunk.x * CHUNK_BLOCKS as i32 + local_x as i32;
			let world_z = chunk.z * CHUNK_BLOCKS as i32 + local_z as i32;
			let Some((y, state)) = surface_top(world, colours, world_x, world_z) else { continue };
			let biome = chunk.biome_at(local_x, y, local_z).map_or(&Biome::PLAINS, |id| biomes.get(id));
			let factor = shade_factor(y) * slope_factor(y, top_height(world, colours, world_x, world_z - 1));
			let (sprite, tint) = colours.texels(state, biome, world_x, world_z);
			let left = ((offset_x as usize * CHUNK_BLOCKS + local_x) * SPRITE_SIZE as usize) as u32;
			let top = ((offset_z as usize * CHUNK_BLOCKS + local_z) * SPRITE_SIZE as usize) as u32;
			paint(image, left, top, sprite, tint, factor);
		}
	}
}

fn paint(image: &mut RgbaImage, left: u32, top: u32, sprite: &RgbaImage, tint: [u8; 3], shade: f32) {
	for (x, y, texel) in sprite.enumerate_pixels() {
		let colour: [u8; 3] = std::array::from_fn(|channel| (f32::from(texel[channel]) * f32::from(tint[channel]) / 255.0 * shade).min(255.0) as u8);
		image.put_pixel(left + x, top + y, Rgba([colour[0], colour[1], colour[2], 255]));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::biome::BiomeRegistry;
	use crate::test_support::{chunk_nbt, region_bytes, stone_assets};

	#[test]
	fn draws_each_block_with_its_texture_at_sprite_size_and_height_shade() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("r.0.0.mca");
		let nbt = chunk_nbt(0, 0, 3955, &[((0, 0, 0), "minecraft:stone")]);
		std::fs::write(&path, region_bytes(&[(0, 0, nbt)])).unwrap();
		let region = RegionFile::open(&path).unwrap();
		let assets = stone_assets();
		let biomes = BiomeRegistry::default();
		let mut colours = BlockColours::new(&assets, &biomes);
		let image = render_detail_tile(&region, 0, 0, &BlockDefaults::default(), &mut colours).unwrap();
		assert_eq!(image.dimensions(), (TILE_PIXELS, TILE_PIXELS));
		let lit = (120.0 * shade_factor(0)) as u8;
		assert_eq!(image.get_pixel(SPRITE_SIZE - 1, SPRITE_SIZE - 1), &Rgba([lit, lit, lit, 255]));
		assert_eq!(image.get_pixel(SPRITE_SIZE, 0)[3], 0);
	}
}
