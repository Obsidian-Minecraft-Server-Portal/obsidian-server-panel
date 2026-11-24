mod settings;

use std::path::Path;

use obsidian_common::Result;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use settings::{
	ConfigEntry, EntryKind, MapSettings, MapSettingsUpdate, accept_download, apply_settings, port_path, read_download_accepted, read_settings,
	scalar_entries, set_scalar_entry,
};

const BLUEMAP_CORE_CONFIG: &str = "core.conf";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MapKind {
	Bluemap,
	Dynmap,
	Squaremap,
	Pl3xmap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layout {
	Hocon,
	DynmapYaml,
	NestedYaml,
}

impl MapKind {
	pub const ALL: [MapKind; 4] = [MapKind::Bluemap, MapKind::Dynmap, MapKind::Squaremap, MapKind::Pl3xmap];

	pub fn label(self) -> &'static str {
		match self {
			Self::Bluemap => "BlueMap",
			Self::Dynmap => "Dynmap",
			Self::Squaremap => "squaremap",
			Self::Pl3xmap => "Pl3xMap",
		}
	}

	pub fn default_port(self) -> u16 {
		match self {
			Self::Bluemap => 8100,
			Self::Dynmap => 8123,
			Self::Squaremap | Self::Pl3xmap => 8080,
		}
	}

	pub fn editable(self) -> bool {
		self.layout() != Layout::Hocon
	}

	pub(crate) fn layout(self) -> Layout {
		match self {
			Self::Bluemap => Layout::Hocon,
			Self::Dynmap => Layout::DynmapYaml,
			Self::Squaremap | Self::Pl3xmap => Layout::NestedYaml,
		}
	}

	pub fn key(self) -> &'static str {
		match self {
			Self::Bluemap => "bluemap",
			Self::Dynmap => "dynmap",
			Self::Squaremap => "squaremap",
			Self::Pl3xmap => "pl3xmap",
		}
	}

	fn config_candidates(self) -> &'static [&'static str] {
		match self {
			Self::Bluemap => &["plugins/BlueMap/webserver.conf", "config/bluemap/webserver.conf"],
			Self::Dynmap => &["plugins/dynmap/configuration.txt", "dynmap/configuration.txt"],
			Self::Squaremap => &["plugins/squaremap/config.yml", "squaremap/config.yml"],
			Self::Pl3xmap => &["plugins/Pl3xMap/config.yml", "config/pl3xmap/config.yml"],
		}
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DetectedMap {
	pub kind: MapKind,
	pub label: &'static str,
	/// Config file relative to the server folder, using forward slashes. None until the map has written its config.
	pub config_path: Option<String>,
	pub settings: MapSettings,
	pub editable: bool,
	/// BlueMap only: whether core.conf already lets it download the Minecraft client file. None when unknown.
	pub download_accepted: Option<bool>,
	/// Set when the config exists but cannot be read; the settings then show the defaults.
	pub error: Option<String>,
}

pub async fn detect(server_dir: &Path) -> Result<Vec<DetectedMap>> {
	let jars = jar_names(server_dir).await?;
	let mut maps = Vec::new();
	for kind in MapKind::ALL {
		if !jars.iter().any(|jar| jar.contains(kind.key())) {
			continue;
		}
		let found = find_config(server_dir, kind).await?;
		let download_accepted = match kind {
			MapKind::Bluemap => bluemap_core_config(server_dir).await?.and_then(|(_, text)| read_download_accepted(&text)),
			MapKind::Dynmap | MapKind::Squaremap | MapKind::Pl3xmap => None,
		};
		let (config_path, settings, error) = match found {
			Some((relative, text)) => match read_settings(kind, &text) {
				Ok(settings) => (Some(relative), settings, None),
				Err(error) => (Some(relative), MapSettings::defaults(kind), Some(error.to_string())),
			},
			None => (None, MapSettings::defaults(kind), None),
		};
		maps.push(DetectedMap {
			kind,
			label: kind.label(),
			config_path,
			settings,
			editable: kind.editable(),
			download_accepted,
			error,
		});
	}
	Ok(maps)
}

pub async fn config_text(server_dir: &Path, kind: MapKind) -> Result<Option<(String, String)>> {
	find_config(server_dir, kind).await
}

async fn jar_names(server_dir: &Path) -> Result<Vec<String>> {
	let mut names = Vec::new();
	for folder in ["plugins", "mods"] {
		let Ok(mut entries) = tokio::fs::read_dir(server_dir.join(folder)).await else { continue };
		while let Some(entry) = entries.next_entry().await? {
			let name = entry.file_name().to_string_lossy().to_lowercase();
			if name.ends_with(".jar") {
				names.push(name);
			}
		}
	}
	Ok(names)
}

async fn find_config(server_dir: &Path, kind: MapKind) -> Result<Option<(String, String)>> {
	for relative in kind.config_candidates() {
		if let Some(text) = read_optional(&server_dir.join(relative)).await? {
			return Ok(Some(((*relative).to_owned(), text)));
		}
	}
	Ok(None)
}

async fn read_optional(path: &Path) -> Result<Option<String>> {
	match tokio::fs::read_to_string(path).await {
		Ok(text) => Ok(Some(text)),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
		Err(error) => Err(error.into()),
	}
}

pub async fn bluemap_core_config(server_dir: &Path) -> Result<Option<(String, String)>> {
	let Some((webserver, _)) = find_config(server_dir, MapKind::Bluemap).await? else {
		return Ok(None);
	};
	let relative = Path::new(&webserver).with_file_name(BLUEMAP_CORE_CONFIG).to_string_lossy().into_owned();
	let text = read_optional(&server_dir.join(&relative)).await?;
	Ok(text.map(|text| (relative, text)))
}
