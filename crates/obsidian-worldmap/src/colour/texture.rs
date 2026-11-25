use image::{RgbaImage, imageops, imageops::FilterType};

use crate::error::Result;

/// A block texture in two forms: its alpha-weighted mean colour, and its top frame at sprite size with the
/// transparent texels blended into that mean, so a sprite is always opaque.
#[derive(Clone)]
pub struct Decoded {
	pub average: [u8; 3],
	pub sprite: RgbaImage,
}

/// `None` when every pixel of the texture is transparent.
pub fn decode(png: &[u8], size: u32) -> Result<Option<Decoded>> {
	let image = image::load_from_memory(png)?.to_rgba8();
	let Some(average) = mean(&image) else { return Ok(None) };
	let side = image.width().min(image.height());
	let frame = imageops::crop_imm(&image, 0, 0, side, side).to_image();
	let mut sprite = imageops::resize(&frame, size, size, FilterType::Triangle);
	for pixel in sprite.pixels_mut() {
		let alpha = f32::from(pixel[3]) / 255.0;
		for channel in 0..3 {
			pixel[channel] = (f32::from(pixel[channel]) * alpha + f32::from(average[channel]) * (1.0 - alpha)).round() as u8;
		}
		pixel[3] = 255;
	}
	Ok(Some(Decoded { average, sprite }))
}

/// A vanilla colormap, indexed by temperature (left to right) and humidity (bottom to top).
pub struct Colormap(RgbaImage);

impl Colormap {
	pub fn decode(png: &[u8]) -> Result<Self> {
		Ok(Self(image::load_from_memory(png)?.to_rgba8()))
	}

	/// Humidity is scaled by temperature before sampling, as vanilla does.
	pub fn sample(&self, temperature: f32, humidity: f32) -> [u8; 3] {
		let image = &self.0;
		let temperature = temperature.clamp(0.0, 1.0);
		let humidity = humidity.clamp(0.0, 1.0) * temperature;
		let x = (((1.0 - temperature) * (image.width() - 1) as f32) as u32).min(image.width() - 1);
		let y = (((1.0 - humidity) * (image.height() - 1) as f32) as u32).min(image.height() - 1);
		let pixel = image.get_pixel(x, y);
		[pixel[0], pixel[1], pixel[2]]
	}
}

fn mean(image: &RgbaImage) -> Option<[u8; 3]> {
	let mut sum = [0.0_f64; 3];
	let mut weight = 0.0_f64;
	for pixel in image.pixels() {
		let alpha = f64::from(pixel[3]) / 255.0;
		for (channel, total) in sum.iter_mut().enumerate() {
			*total += f64::from(pixel[channel]) * alpha;
		}
		weight += alpha;
	}
	(weight > 0.0).then(|| sum.map(|total| (total / weight).round() as u8))
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use image::{ImageFormat, Rgba};

	use super::*;

	fn png(pixels: &[[u8; 4]], width: u32) -> Vec<u8> {
		let height = pixels.len() as u32 / width;
		let mut image = RgbaImage::new(width, height);
		for (index, pixel) in pixels.iter().enumerate() {
			image.put_pixel(index as u32 % width, index as u32 / width, Rgba(*pixel));
		}
		let mut bytes = Vec::new();
		image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).unwrap();
		bytes
	}

	#[test]
	fn averages_only_the_visible_pixels() {
		let bytes = png(&[[200, 100, 0, 255], [0, 0, 0, 0]], 2);
		assert_eq!(decode(&bytes, 8).unwrap().map(|decoded| decoded.average), Some([200, 100, 0]));
	}

	#[test]
	fn reports_no_colour_for_fully_transparent_textures() {
		assert!(decode(&png(&[[9, 9, 9, 0]], 1), 8).unwrap().is_none());
	}

	#[test]
	fn the_sprite_is_the_top_frame_scaled_to_size() {
		let top_red = [[255, 0, 0, 255], [255, 0, 0, 255], [255, 0, 0, 255], [255, 0, 0, 255]];
		let below_blue = [[0, 0, 255, 255], [0, 0, 255, 255], [0, 0, 255, 255], [0, 0, 255, 255]];
		let frames = [top_red, below_blue].concat();
		let sprite = decode(&png(&frames, 2), 8).unwrap().unwrap().sprite;
		assert_eq!(sprite.dimensions(), (8, 8));
		assert!(sprite.pixels().all(|pixel| *pixel == Rgba([255, 0, 0, 255])));
	}

	#[test]
	fn transparent_texels_take_the_average_colour() {
		let bytes = png(&[[200, 100, 0, 255], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]], 2);
		let sprite = decode(&bytes, 8).unwrap().unwrap().sprite;
		assert_eq!(sprite.get_pixel(7, 7), &Rgba([200, 100, 0, 255]));
	}

	#[test]
	fn samples_colormaps_by_temperature_and_humidity() {
		let bytes = png(&[[1, 2, 3, 255], [4, 5, 6, 255], [7, 8, 9, 255], [10, 11, 12, 255]], 2);
		let colormap = Colormap::decode(&bytes).unwrap();
		assert_eq!(colormap.sample(0.0, 0.0), [10, 11, 12]);
		assert_eq!(colormap.sample(1.0, 1.0), [1, 2, 3]);
	}
}
