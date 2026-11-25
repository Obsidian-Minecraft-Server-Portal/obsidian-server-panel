//! Player skins: the default Steve and Alex skins a client jar ships, the arm model a skin is drawn with, and the skin
//! URL inside Mojang's profile.

use std::path::Path;

use serde::Deserialize;

use crate::error::{Result, WorldmapError};

const TEXTURE_HOST: &str = "textures.minecraft.net/";
const SKIN_SIZES: [(u32, u32); 2] = [(64, 64), (64, 32)];

/// The arm model a skin is drawn with: wide arms are four pixels across, slim arms three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
	Wide,
	Slim,
}

impl Model {
	pub fn name(self) -> &'static str {
		match self {
			Self::Wide => "wide",
			Self::Slim => "slim",
		}
	}
}

/// A skin a profile points at, and the model it is drawn with.
#[derive(Debug, PartialEq, Eq)]
pub struct SkinRef {
	pub url: String,
	pub model: Model,
}

/// The default skin a client jar ships: Steve with wide arms, or Alex with slim arms. `None` when the jar lacks it.
pub fn default_skin_png(client_jar: &Path, alex: bool) -> Result<Option<Vec<u8>>> {
	let (name, model) = if alex { ("alex", "slim") } else { ("steve", "wide") };
	let candidates = [
		format!("assets/minecraft/textures/entity/player/{model}/{name}.png"),
		format!("assets/minecraft/textures/entity/{name}.png"),
	];
	for candidate in candidates {
		if let Some(skin) = crate::assets::read_entry(client_jar, &candidate)? {
			return Ok(Some(skin));
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

/// The skin URL and model inside a profile's `textures` property, once the property is base64-decoded.
pub fn skin_ref(textures_json: &[u8]) -> Option<SkinRef> {
	let profile: Profile = serde_json::from_slice(textures_json).ok()?;
	let skin = profile.textures.skin?;
	let rest = skin.url.strip_prefix("https://").or_else(|| skin.url.strip_prefix("http://"))?;
	if !rest.starts_with(TEXTURE_HOST) {
		return None;
	}
	let model = match skin.metadata.and_then(|metadata| metadata.model).as_deref() {
		Some("slim") => Model::Slim,
		_ => Model::Wide,
	};
	Some(SkinRef {
		url: format!("https://{rest}"),
		model,
	})
}

/// Rejects anything that is not a player skin, so a cached skin is always one the viewer can draw.
pub fn check_skin(png: &[u8]) -> Result<()> {
	let skin = image::load_from_memory(png)?;
	let size = (skin.width(), skin.height());
	if SKIN_SIZES.contains(&size) {
		Ok(())
	} else {
		Err(WorldmapError::Asset(format!("a {}x{} image is not a player skin", size.0, size.1)))
	}
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
	metadata: Option<Metadata>,
}

#[derive(Deserialize)]
struct Metadata {
	model: Option<String>,
}

#[cfg(test)]
mod tests {
	use image::RgbaImage;

	use super::*;
	use crate::tiles::encode_png;

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
	fn skin_urls_come_only_from_mojang_texture_hosts() {
		let wide = br#"{"textures": {"SKIN": {"url": "https://textures.minecraft.net/texture/abc"}}}"#;
		assert_eq!(
			skin_ref(wide),
			Some(SkinRef {
				url: "https://textures.minecraft.net/texture/abc".into(),
				model: Model::Wide,
			})
		);
		let slim = br#"{"textures": {"SKIN": {"url": "https://textures.minecraft.net/texture/def", "metadata": {"model": "slim"}}}}"#;
		assert_eq!(skin_ref(slim).map(|skin| skin.model), Some(Model::Slim));
		let plain = br#"{"textures": {"SKIN": {"url": "http://textures.minecraft.net/texture/abc"}}}"#;
		assert_eq!(skin_ref(plain).map(|skin| skin.url).as_deref(), Some("https://textures.minecraft.net/texture/abc"));
		let elsewhere = br#"{"textures": {"SKIN": {"url": "https://example.com/skin.png"}}}"#;
		assert_eq!(skin_ref(elsewhere), None);
		let lookalike = br#"{"textures": {"SKIN": {"url": "http://textures.minecraft.net.example.com/skin.png"}}}"#;
		assert_eq!(skin_ref(lookalike), None);
		assert_eq!(skin_ref(br#"{"textures": {}}"#), None);
	}

	#[test]
	fn only_player_skin_sizes_are_accepted() {
		assert!(check_skin(&encode_png(&RgbaImage::new(64, 64)).unwrap()).is_ok());
		assert!(check_skin(&encode_png(&RgbaImage::new(64, 32)).unwrap()).is_ok());
		assert!(check_skin(&encode_png(&RgbaImage::new(16, 16)).unwrap()).is_err());
	}
}
