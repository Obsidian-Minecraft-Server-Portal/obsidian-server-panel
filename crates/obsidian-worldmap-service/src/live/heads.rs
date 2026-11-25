//! Heads for the live players: each player's skin face, fetched from Mojang once, and the default Steve and Alex heads
//! for players whose skin is not known.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use obsidian_common::{CommonError, Result};
use obsidian_worldmap::skins;
use serde::Deserialize;

const PROFILE_URL: &str = "https://sessionserver.mojang.com/session/minecraft/profile/";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
pub const STEVE: &str = "steve.png";
pub const ALEX: &str = "alex.png";

#[derive(Clone)]
pub struct Heads {
	dir: PathBuf,
	http: reqwest::Client,
}

impl Heads {
	pub fn new(cache_dir: &Path) -> Result<Self> {
		let dir = cache_dir.join("worldmap").join("heads");
		std::fs::create_dir_all(&dir)?;
		let http = reqwest::Client::builder()
			.timeout(REQUEST_TIMEOUT)
			.user_agent("obsidian-worldmap")
			.build()
			.map_err(other)?;
		Ok(Self { dir, http })
	}

	pub fn path(&self, file: &str) -> PathBuf {
		self.dir.join(file)
	}

	pub fn has(&self, file: &str) -> bool {
		self.dir.join(file).is_file()
	}

	/// The head a player shows: their skin's head once it is cached, otherwise the default head for their UUID.
	pub fn file_for(&self, uuid: Option<&str>) -> String {
		match uuid {
			Some(uuid) if self.has(&skin_file(uuid)) => skin_file(uuid),
			Some(uuid) if skins::default_is_alex(uuid) => ALEX.to_owned(),
			_ => STEVE.to_owned(),
		}
	}

	/// Writes the Steve and Alex heads from the client jar, unless they are already cached or the jar lacks them.
	pub async fn ensure_defaults(&self, client_jar: &Path) -> Result<()> {
		for (file, alex) in [(STEVE, false), (ALEX, true)] {
			if self.has(file) {
				continue;
			}
			let jar = client_jar.to_owned();
			let head = tokio::task::spawn_blocking(move || skins::default_head_png(&jar, alex))
				.await
				.map_err(other)?
				.map_err(other)?;
			if let Some(png) = head {
				tokio::fs::write(self.path(file), png).await?;
			}
		}
		Ok(())
	}

	/// Downloads a player's skin and keeps its head. Returns `false` when Mojang has no skin for the player.
	pub async fn fetch_skin(&self, uuid: &str) -> Result<bool> {
		let profile: Profile = self
			.http
			.get(format!("{PROFILE_URL}{}", uuid.replace('-', "")))
			.send()
			.await
			.and_then(|response| response.error_for_status())
			.map_err(other)?
			.json()
			.await
			.map_err(other)?;
		let Some(textures) = profile.properties.into_iter().find(|property| property.name == "textures") else {
			return Ok(false);
		};
		let decoded = STANDARD.decode(textures.value).map_err(other)?;
		let Some(url) = skins::skin_url(&decoded) else { return Ok(false) };
		let skin = self
			.http
			.get(url)
			.send()
			.await
			.and_then(|response| response.error_for_status())
			.map_err(other)?
			.bytes()
			.await
			.map_err(other)?;
		let head = skins::head_png(&skin).map_err(other)?;
		tokio::fs::write(self.path(&skin_file(uuid)), head).await?;
		Ok(true)
	}
}

/// Head file names are lowercase hex UUIDs with a `.png` extension, or one of the defaults.
pub fn is_head_file(name: &str) -> bool {
	name.strip_suffix(".png")
		.is_some_and(|stem| !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase()))
}

pub fn skin_file(uuid: &str) -> String {
	format!("{}.png", uuid.replace('-', "").to_ascii_lowercase())
}

/// The players the server has seen, by lowercase name, with dashed UUIDs.
pub fn user_cache(instance_dir: &Path) -> HashMap<String, String> {
	#[derive(Deserialize)]
	struct Entry {
		name: String,
		uuid: String,
	}
	let Ok(bytes) = std::fs::read(instance_dir.join("usercache.json")) else { return HashMap::new() };
	let entries: Vec<Entry> = serde_json::from_slice(&bytes).unwrap_or_default();
	entries.into_iter().map(|entry| (entry.name.to_ascii_lowercase(), entry.uuid)).collect()
}

#[derive(Deserialize)]
struct Profile {
	#[serde(default)]
	properties: Vec<Property>,
}

#[derive(Deserialize)]
struct Property {
	name: String,
	value: String,
}

fn other(error: impl Display) -> CommonError {
	CommonError::Other(error.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn only_head_cache_names_are_served() {
		assert!(is_head_file("steve.png"));
		assert!(is_head_file("0123456789abcdef0123456789abcdef.png"));
		assert!(!is_head_file("../secret.png"));
		assert!(!is_head_file("Steve.png"));
		assert!(!is_head_file(".png"));
	}

	#[test]
	fn skin_files_drop_the_dashes() {
		assert_eq!(skin_file("853C80EF-3C37-49FD-AA49-938B674ADAE6"), "853c80ef3c3749fdaa49938b674adae6.png");
	}

	#[test]
	fn the_user_cache_maps_lowercase_names_to_uuids() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(
			dir.path().join("usercache.json"),
			br#"[{"name": "Steve", "uuid": "853c80ef-3c37-49fd-aa49-938b674adae6", "expiresOn": "2030-01-01 00:00:00 +0000"}]"#,
		)
		.unwrap();
		let cache = user_cache(dir.path());
		assert_eq!(cache.get("steve").map(String::as_str), Some("853c80ef-3c37-49fd-aa49-938b674adae6"));
		assert!(user_cache(&dir.path().join("missing")).is_empty());
	}
}
