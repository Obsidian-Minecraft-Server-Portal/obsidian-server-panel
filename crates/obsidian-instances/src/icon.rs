use std::io::Cursor;

use image::imageops::FilterType;
use image::ImageFormat;
use obsidian_common::{CommonError, InstanceId, Result};

use crate::manager::InstanceManager;

const ICON_FILE: &str = "server-icon.png";
const ICON_SIZE: u32 = 64;

fn encode_icon(bytes: &[u8]) -> Result<Vec<u8>> {
	let source = image::load_from_memory(bytes).map_err(|error| CommonError::Invalid(format!("unsupported image: {error}")))?;
	let icon = source.resize_to_fill(ICON_SIZE, ICON_SIZE, FilterType::Lanczos3);
	let mut encoded = Cursor::new(Vec::new());
	icon.write_to(&mut encoded, ImageFormat::Png)
		.map_err(|error| CommonError::Other(format!("failed to encode server icon: {error}")))?;
	Ok(encoded.into_inner())
}

impl InstanceManager {
	pub async fn set_icon(&self, id: InstanceId, bytes: Vec<u8>) -> Result<()> {
		let instance = self.get(id).await?;
		let png = tokio::task::spawn_blocking(move || encode_icon(&bytes))
			.await
			.map_err(|error| CommonError::Other(error.to_string()))??;
		let path = instance.directory.join(ICON_FILE);
		tokio::fs::write(&path, png).await?;
		sqlx::query("UPDATE instances SET icon_path = ?, updated_at = unixepoch() WHERE id = ?")
			.bind(path.to_string_lossy())
			.bind(id)
			.execute(self.db.pool())
			.await?;
		Ok(())
	}
}
