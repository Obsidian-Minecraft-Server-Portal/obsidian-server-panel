use image::RgbaImage;

use crate::error::Result;

/// Alpha-weighted mean colour of a PNG, or `None` when every pixel is transparent.
pub fn average(png: &[u8]) -> Result<Option<[u8; 3]>> {
	Ok(mean(&image::load_from_memory(png)?.to_rgba8()))
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
		assert_eq!(average(&bytes).unwrap(), Some([200, 100, 0]));
	}

	#[test]
	fn reports_no_colour_for_fully_transparent_textures() {
		assert_eq!(average(&png(&[[9, 9, 9, 0]], 1)).unwrap(), None);
	}

	#[test]
	fn samples_colormaps_by_temperature_and_humidity() {
		let bytes = png(&[[1, 2, 3, 255], [4, 5, 6, 255], [7, 8, 9, 255], [10, 11, 12, 255]], 2);
		let colormap = Colormap::decode(&bytes).unwrap();
		assert_eq!(colormap.sample(0.0, 0.0), [10, 11, 12]);
		assert_eq!(colormap.sample(1.0, 1.0), [1, 2, 3]);
	}
}
