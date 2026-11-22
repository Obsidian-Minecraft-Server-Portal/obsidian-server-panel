use serde_json::Value;

use crate::archive::{self, Archive};

const MAX_ICON_BYTES: u64 = 1 << 20;

pub fn extract(jar: &[u8]) -> Option<Vec<u8>> {
	let mut archive = archive::open(jar).ok()?;
	let path = icon_path(&mut archive)?;
	archive::read(&mut archive, path.trim_start_matches('/'), MAX_ICON_BYTES)
}

pub fn mime(bytes: &[u8]) -> &'static str {
	match bytes {
		[0x89, b'P', b'N', b'G', ..] => "image/png",
		[0xFF, 0xD8, 0xFF, ..] => "image/jpeg",
		[b'G', b'I', b'F', b'8', ..] => "image/gif",
		[b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "image/webp",
		_ => "application/octet-stream",
	}
}

fn icon_path(archive: &mut Archive<'_>) -> Option<String> {
	if let Some(text) = archive::read_text(archive, "fabric.mod.json") {
		let json: Value = serde_json::from_str(&text).ok()?;
		return largest_icon(json.get("icon")?);
	}
	if let Some(text) = archive::read_text(archive, "quilt.mod.json") {
		let json: Value = serde_json::from_str(&text).ok()?;
		return largest_icon(json.pointer("/quilt_loader/metadata/icon")?);
	}
	["META-INF/neoforge.mods.toml", "META-INF/mods.toml"]
		.into_iter()
		.find_map(|name| {
			let table: toml::Table = toml::from_str(&archive::read_text(archive, name)?).ok()?;
			table.get("logoFile")?.as_str().map(str::to_owned)
		})
}

fn largest_icon(value: &Value) -> Option<String> {
	match value {
		Value::String(path) => Some(path.clone()),
		Value::Object(sizes) => sizes
			.iter()
			.filter_map(|(size, path)| Some((size.parse::<u32>().ok()?, path.as_str()?)))
			.max_by_key(|(size, _)| *size)
			.map(|(_, path)| path.to_owned()),
		_ => None,
	}
}
