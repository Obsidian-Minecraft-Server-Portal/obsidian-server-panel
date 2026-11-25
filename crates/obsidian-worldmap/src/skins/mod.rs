//! Player heads: the face of a skin with its hat layer on top, the default skin a player without one wears, and the
//! skin URL inside Mojang's profile.

use std::path::Path;

use image::RgbaImage;
use serde::Deserialize;

use crate::error::{Result, WorldmapError};

const HEAD: u32 = 8;
const FACE_AT: (u32, u32) = (8, 8);
const HAT_AT: (u32, u32) = (40, 8);
const TEXTURE_HOST: &str = "https://textures.minecraft.net/";

/// The 8x8 head of a 64x64 (or 64x32) skin. Hat pixels cover the face wherever they are not transparent.
pub fn head_from_skin(skin: &RgbaImage) -> Result<RgbaImage> {
	if skin.width() < 64 || skin.height() < 32 {
		return Err(WorldmapError::Asset(format!("a {}x{} skin has no head", skin.width(), skin.height())));
	}
	Ok(RgbaImage::from_fn(HEAD, HEAD, |x, y| {
		let hat = *skin.get_pixel(HAT_AT.0 + x, HAT_AT.1 + y);
		if hat[3] > 0 {
			hat
		} else {
			*skin.get_pixel(FACE_AT.0 + x, FACE_AT.1 + y)
		}
	}))
}

/// The PNG of a skin's head, as the head cache stores it.
pub fn head_png(skin_png: &[u8]) -> Result<Vec<u8>> {
	let skin = image::load_from_memory(skin_png)?.to_rgba8();
	crate::tiles::encode_png(&head_from_skin(&skin)?)
}

/// The Steve or Alex head from the default skins in a client jar. `None` when the jar does not have them.
pub fn default_head_png(client_jar: &Path, alex: bool) -> Result<Option<Vec<u8>>> {
	let name = if alex { "alex" } else { "steve" };
	let model = if alex { "slim" } else { "wide" };
	let candidates = [
		format!("assets/minecraft/textures/entity/player/{model}/{name}.png"),
		format!("assets/minecraft/textures/entity/{name}.png"),
	];
	for candidate in candidates {
		if let Some(skin) = crate::assets::read_entry(client_jar, &candidate)? {
			return head_png(&skin).map(Some);
		}
	}
	Ok(None)
}

/// Players without a skin wear Alex when the low bit of their UUID's `hashCode()` is set, and Steve otherwise.
pub fn default_is_alex(uuid: &str) -> bool {
	let Ok(bits) = u128::from_str_radix(&uuid.replace('-', ""), 16) else { return false };
	let hilo = ((bits >> 64) as u64) ^ (bits as u64);
	((hilo >> 32) as u32 ^ hilo as u32) & 1 == 1
}

#[derive(Deserialize)]
struct Profile {
	textures: Textures,
}

#[derive(Deserialize)]
struct Textures {
	#[serde(rename = "SKIN")]
	skin: Option<Skin>,
}

#[derive(Deserialize)]
struct Skin {
	url: String,
}

/// The skin URL in a profile's `textures` property, once the property is base64-decoded.
pub fn skin_url(textures_json: &[u8]) -> Option<String> {
	let profile: Profile = serde_json::from_slice(textures_json).ok()?;
	let url = profile.textures.skin?.url;
	url.starts_with(TEXTURE_HOST).then_some(url)
}

#[cfg(test)]
mod tests {
	use image::Rgba;

	use super::*;

	#[test]
	fn the_hat_layer_covers_the_face_where_it_is_opaque() {
		let mut skin = RgbaImage::from_pixel(64, 64, Rgba([0, 0, 0, 0]));
		for y in 0..8 {
			for x in 0..8 {
				skin.put_pixel(FACE_AT.0 + x, FACE_AT.1 + y, Rgba([200, 0, 0, 255]));
			}
		}
		skin.put_pixel(HAT_AT.0 + 3, HAT_AT.1 + 4, Rgba([0, 200, 0, 255]));
		let head = head_from_skin(&skin).unwrap();
		assert_eq!(head.dimensions(), (8, 8));
		assert_eq!(head.get_pixel(0, 0), &Rgba([200, 0, 0, 255]));
		assert_eq!(head.get_pixel(3, 4), &Rgba([0, 200, 0, 255]));
		assert!(head_from_skin(&RgbaImage::new(16, 16)).is_err());
	}

	#[test]
	fn the_default_skin_follows_the_java_hash_code() {
		// Expected values are UUID.fromString(...).hashCode() & 1, computed on the bundled JDK.
		assert!(default_is_alex("853c80ef-3c37-49fd-aa49-938b674adae6"));
		assert!(default_is_alex("00000000-0000-0000-0000-000000000001"));
		assert!(!default_is_alex("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
		assert!(!default_is_alex("f84c6a79-0a4e-45e0-879b-cd49ebd4c4e2"));
		assert!(!default_is_alex("9be4e7f8-1b0d-4f3a-bd2c-5a6e7f8091a2"));
		assert!(!default_is_alex("not a uuid"));
	}

	#[test]
	fn skin_urls_are_read_only_from_mojang_texture_hosts() {
		let profile = br#"{"textures": {"SKIN": {"url": "https://textures.minecraft.net/texture/abc"}}}"#;
		assert_eq!(skin_url(profile).as_deref(), Some("https://textures.minecraft.net/texture/abc"));
		let elsewhere = br#"{"textures": {"SKIN": {"url": "https://example.com/skin.png"}}}"#;
		assert_eq!(skin_url(elsewhere), None);
		assert_eq!(skin_url(br#"{"textures": {}}"#), None);
	}
}
