use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

use obsidian_common::{CommonError, InstanceId, Result};

use crate::manager::InstanceManager;

const BANNER_WIDTH: u32 = 1280;
const BANNER_HEIGHT: u32 = 360;

fn encode_banner(bytes: &[u8]) -> Result<Vec<u8>> {
	let source = image::load_from_memory(bytes)
		.map_err(|error| CommonError::Invalid(format!("unsupported image: {error}")))?;
	let banner = DynamicImage::ImageRgb8(
		source.resize_to_fill(BANNER_WIDTH, BANNER_HEIGHT, FilterType::Lanczos3).into_rgb8(),
	);
	let mut encoded = Cursor::new(Vec::new());
	banner
		.write_to(&mut encoded, ImageFormat::Jpeg)
		.map_err(|error| CommonError::Other(format!("failed to encode server banner: {error}")))?;
	Ok(encoded.into_inner())
}

impl InstanceManager {
	pub async fn set_banner(&self, id: InstanceId, bytes: Vec<u8>) -> Result<()> {
		let jpeg = tokio::task::spawn_blocking(move || encode_banner(&bytes))
			.await
			.map_err(|error| CommonError::Other(error.to_string()))??;
		let path = self.paths.banners_dir.join(format!("{id}.jpg"));
		tokio::fs::write(&path, jpeg).await?;
		sqlx::query("UPDATE instances SET banner_path = ?, updated_at = unixepoch() WHERE id = ?")
			.bind(path.to_string_lossy())
			.bind(id)
			.execute(self.db.pool())
			.await?;
		Ok(())
	}

	pub(crate) async fn remove_banner(&self, id: InstanceId) {
		let _ = tokio::fs::remove_file(self.paths.banners_dir.join(format!("{id}.jpg"))).await;
	}

	pub async fn clear_banner(&self, id: InstanceId) -> Result<()> {
		self.remove_banner(id).await;
		sqlx::query("UPDATE instances SET banner_path = NULL, updated_at = unixepoch() WHERE id = ?")
			.bind(id)
			.execute(self.db.pool())
			.await?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};

	use super::{BANNER_HEIGHT, BANNER_WIDTH, encode_banner};

	#[test]
	fn banners_are_cover_cropped_jpegs() {
		let source = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(2400, 900, Rgb([120u8, 40, 200])));
		let mut png = Cursor::new(Vec::new());
		source.write_to(&mut png, ImageFormat::Png).unwrap();
		let jpeg = encode_banner(&png.into_inner()).unwrap();
		assert_eq!(image::guess_format(&jpeg).unwrap(), ImageFormat::Jpeg);
		let decoded = image::load_from_memory(&jpeg).unwrap();
		assert_eq!((decoded.width(), decoded.height()), (BANNER_WIDTH, BANNER_HEIGHT));
	}
}
