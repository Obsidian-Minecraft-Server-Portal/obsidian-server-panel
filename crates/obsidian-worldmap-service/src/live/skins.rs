//! Skins for the live players: each player's skin, fetched from Mojang once, and the default Steve and Alex skins for
//! players whose skin is not known yet.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use obsidian_common::{CommonError, Result};
use obsidian_worldmap::skins::{self, Model};
use serde::Deserialize;

const PROFILE_URL: &str = "https://sessionserver.mojang.com/session/minecraft/profile/";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
pub const STEVE: &str = "steve.png";
pub const ALEX: &str = "alex.png";

#[derive(Clone)]
pub struct Skins {
	dir: PathBuf,
	http: reqwest::Client,
}

impl Skins {
	pub fn new(cache_dir: &Path) -> Result<Self> {
		let worldmap = cache_dir.join("worldmap");
		let _ = std::fs::remove_dir_all(worldmap.join("heads"));
		let dir = worldmap.join("skins");
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

	fn has(&self, file: &str) -> bool {
		self.dir.join(file).is_file()
	}

	pub fn has_skin(&self, uuid: &str) -> bool {
		[Model::Wide, Model::Slim].into_iter().any(|model| self.has(&skin_file(uuid, model)))
	}

	/// The skin a player shows: their own once it is cached, otherwise the default for their UUID.
	pub fn skin_for(&self, uuid: Option<&str>) -> String {
		let Some(uuid) = uuid else { return STEVE.to_owned() };
		[Model::Wide, Model::Slim]
			.into_iter()
			.map(|model| skin_file(uuid, model))
			.find(|file| self.has(file))
			.unwrap_or_else(|| if skins::default_is_alex(uuid) { ALEX.to_owned() } else { STEVE.to_owned() })
	}

	/// Writes the Steve and Alex skins from the client jar, unless they are already cached or the jar lacks them.
	pub async fn ensure_defaults(&self, client_jar: &Path) -> Result<()> {
		for (file, alex) in [(STEVE, false), (ALEX, true)] {
			if self.has(file) {
				continue;
			}
			let jar = client_jar.to_owned();
			let png = tokio::task::spawn_blocking(move || skins::default_skin_png(&jar, alex))
				.await
				.map_err(other)?
				.map_err(other)?;
			if let Some(png) = png {
				tokio::fs::write(self.path(file), png).await?;
			}
		}
		Ok(())
	}

	/// Downloads a player's skin and keeps it under the file name of its model. Returns `false` when Mojang has no skin.
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
		let Some(skin) = skins::skin_ref(&decoded) else { return Ok(false) };
		let png = self
			.http
			.get(&skin.url)
			.send()
			.await
			.and_then(|response| response.error_for_status())
			.map_err(other)?
			.bytes()
			.await
			.map_err(other)?;
		skins::check_skin(&png).map_err(other)?;
		let previous = if skin.model == Model::Slim { Model::Wide } else { Model::Slim };
		let _ = tokio::fs::remove_file(self.path(&skin_file(uuid, previous))).await;
		tokio::fs::write(self.path(&skin_file(uuid, skin.model)), &png).await?;
		Ok(true)
	}
}

/// Skin file names are lowercase hex UUIDs with a `.png` extension, a `-slim` suffix for slim skins, or a default.
pub fn is_skin_file(name: &str) -> bool {
	name.strip_suffix(".png")
		.map(|stem| stem.strip_suffix("-slim").unwrap_or(stem))
		.is_some_and(|stem| !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase()))
}

pub fn skin_file(uuid: &str, model: Model) -> String {
	let hex = uuid.replace('-', "").to_ascii_lowercase();
	match model {
		Model::Wide => format!("{hex}.png"),
		Model::Slim => format!("{hex}-slim.png"),
	}
}

/// The arm model a skin file is drawn with.
pub fn model_of(file: &str) -> Model {
	if file == ALEX || file.ends_with("-slim.png") {
		Model::Slim
	} else {
		Model::Wide
	}
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
	fn only_skin_cache_names_are_served() {
		assert!(is_skin_file("steve.png"));
		assert!(is_skin_file("0123456789abcdef0123456789abcdef.png"));
		assert!(is_skin_file("0123456789abcdef0123456789abcdef-slim.png"));
		assert!(!is_skin_file("../secret.png"));
		assert!(!is_skin_file("Steve.png"));
		assert!(!is_skin_file(".png"));
		assert!(!is_skin_file("-slim.png"));
	}

	#[test]
	fn skin_files_drop_the_dashes_and_mark_slim_skins() {
		let uuid = "853C80EF-3C37-49FD-AA49-938B674ADAE6";
		assert_eq!(skin_file(uuid, Model::Wide), "853c80ef3c3749fdaa49938b674adae6.png");
		assert_eq!(skin_file(uuid, Model::Slim), "853c80ef3c3749fdaa49938b674adae6-slim.png");
	}

	#[test]
	fn the_model_follows_the_file_name() {
		assert_eq!(model_of("alex.png"), Model::Slim);
		assert_eq!(model_of("steve.png"), Model::Wide);
		assert_eq!(model_of("853c80ef3c3749fdaa49938b674adae6-slim.png"), Model::Slim);
		assert_eq!(model_of("853c80ef3c3749fdaa49938b674adae6.png"), Model::Wide);
	}

	#[test]
	fn a_players_own_skin_wins_over_the_default() {
		let dir = tempfile::tempdir().unwrap();
		let skins = Skins::new(dir.path()).unwrap();
		let uuid = "853c80ef-3c37-49fd-aa49-938b674adae6";
		assert_eq!(skins.skin_for(Some(uuid)), ALEX);
		std::fs::write(skins.path(&skin_file(uuid, Model::Wide)), b"skin").unwrap();
		assert_eq!(skins.skin_for(Some(uuid)), skin_file(uuid, Model::Wide));
		assert_eq!(skins.skin_for(None), STEVE);
	}

	#[test]
	fn the_head_cache_from_earlier_builds_is_discarded() {
		let dir = tempfile::tempdir().unwrap();
		let heads = dir.path().join("worldmap").join("heads");
		std::fs::create_dir_all(&heads).unwrap();
		std::fs::write(heads.join("steve.png"), b"old head").unwrap();
		let skins = Skins::new(dir.path()).unwrap();
		assert!(!heads.exists());
		assert!(skins.path(STEVE).parent().is_some_and(|parent| parent.is_dir()));
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
