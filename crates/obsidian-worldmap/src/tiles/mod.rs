mod colour;

use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};

use crate::chunk::Chunk;
use crate::error::Result;
use crate::region::{CHUNKS_PER_SIDE, RegionFile};

const CHUNK_PIXELS: i32 = 16;
pub const REGION_PIXELS: u32 = CHUNKS_PER_SIDE as u32 * CHUNK_PIXELS as u32;

/// Top-down image of one region at one pixel per block. Columns without a generated chunk stay transparent.
pub fn render_region(region: &RegionFile) -> Result<RgbaImage> {
	let (region_x, region_z) = region.region_coords();
	let mut image = RgbaImage::new(REGION_PIXELS, REGION_PIXELS);
	for local_z in 0..CHUNKS_PER_SIDE {
		for local_x in 0..CHUNKS_PER_SIDE {
			if let Some(nbt) = region.chunk_nbt(local_x, local_z)? {
				let chunk = Chunk::parse(&nbt)?;
				let origin_x = region_x * CHUNKS_PER_SIDE as i32;
				let origin_z = region_z * CHUNKS_PER_SIDE as i32;
				draw_chunk(&mut image, &chunk, origin_x, origin_z);
			}
		}
	}
	Ok(image)
}

fn draw_chunk(image: &mut RgbaImage, chunk: &Chunk, origin_x: i32, origin_z: i32) {
	let base_x = (chunk.x - origin_x) * CHUNK_PIXELS;
	let base_z = (chunk.z - origin_z) * CHUNK_PIXELS;
	let extent = 0..REGION_PIXELS as i32;
	for local_z in 0..CHUNK_PIXELS {
		for local_x in 0..CHUNK_PIXELS {
			let (px, pz) = (base_x + local_x, base_z + local_z);
			if !(extent.contains(&px) && extent.contains(&pz)) {
				continue;
			}
			if let Some((y, state)) = chunk.column_top(local_x as usize, local_z as usize) {
				let [r, g, b] = colour::shade(colour::placeholder(&state.name), y);
				image.put_pixel(px as u32, pz as u32, Rgba([r, g, b, 255]));
			}
		}
	}
}

pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
	let mut png = Vec::new();
	image.write_to(&mut Cursor::new(&mut png), ImageFormat::Png)?;
	Ok(png)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_support::chunk_nbt;

	#[test]
	fn draws_the_top_block_with_height_shading() {
		let nbt = chunk_nbt(0, 0, 3955, &[((0, 0, 0), "minecraft:stone"), ((0, 1, 0), "minecraft:grass_block")]);
		let chunk = Chunk::parse(&nbt).unwrap();
		let mut image = RgbaImage::new(REGION_PIXELS, REGION_PIXELS);
		draw_chunk(&mut image, &chunk, 0, 0);
		let [r, g, b] = colour::shade(colour::placeholder("minecraft:grass_block"), 1);
		assert_eq!(image.get_pixel(0, 0), &Rgba([r, g, b, 255]));
		assert_eq!(image.get_pixel(1, 0)[3], 0);
	}

	#[test]
	fn encodes_a_png_that_decodes_back_to_the_region_size() {
		let png = encode_png(&RgbaImage::new(REGION_PIXELS, REGION_PIXELS)).unwrap();
		let decoded = image::load_from_memory(&png).unwrap();
		assert_eq!((decoded.width(), decoded.height()), (REGION_PIXELS, REGION_PIXELS));
	}
}
