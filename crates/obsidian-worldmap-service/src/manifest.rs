//! What the viewer reads: one dimension's section list and the shared atlas, plus the checks that keep reads inside the map folder.

use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use obsidian_common::{CommonError, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldMapManifest {
	pub pack_key: String,
	pub atlas: String,
	pub atlas_width: u32,
	pub atlas_height: u32,
	/// Changes whenever a render rewrites the block or biome table. The browser keys its copies of them by it.
	pub tables_version: String,
	pub sections: Vec<WorldMapSection>,
	pub tiles: Vec<WorldMapTile>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorldMapSection {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorldMapTile {
	pub x: i32,
	pub z: i32,
	pub file: String,
	/// Unix milliseconds when the region was drawn. Cached copies of the tile and its sections are keyed by it.
	pub version: i64,
}

#[derive(Deserialize)]
struct PackFile {
	#[serde(rename = "packKey")]
	pack_key: String,
	atlas: String,
	#[serde(rename = "atlasWidth")]
	atlas_width: u32,
	#[serde(rename = "atlasHeight")]
	atlas_height: u32,
}

#[derive(Deserialize)]
struct RegionFile {
	x: i32,
	z: i32,
	tile: String,
	sections: Vec<RegionSection>,
	#[serde(rename = "renderedAt", default)]
	rendered_at: i64,
}

#[derive(Deserialize)]
struct RegionSection {
	x: i32,
	y: i32,
	z: i32,
}

#[derive(Deserialize)]
struct RegionStamp {
	#[serde(rename = "renderedAt", default)]
	rendered_at: i64,
}

pub async fn dimension(map_dir: &Path, folder: &str) -> Result<WorldMapManifest> {
	let folder_dir = file_path(map_dir, folder)?;
	let pack: PackFile = read_json(&map_dir.join("pack").join("manifest.json")).await?;
	let mut entries = tokio::fs::read_dir(folder_dir.join("regions"))
		.await
		.map_err(|_| CommonError::NotFound(format!("no render for {folder} yet")))?;
	let mut sections = Vec::new();
	let mut tiles = Vec::new();
	while let Some(entry) = entries.next_entry().await? {
		if entry.path().extension().is_none_or(|extension| extension != "json") {
			continue;
		}
		let region: RegionFile = read_json(&entry.path()).await?;
		tiles.push(WorldMapTile {
			x: region.x,
			z: region.z,
			file: format!("{folder}/{}", region.tile),
			version: region.rendered_at,
		});
		sections.extend(region.sections.into_iter().map(|section| WorldMapSection {
			x: section.x,
			y: section.y,
			z: section.z,
		}));
	}
	sections.sort_by_key(|section| (section.x, section.y, section.z));
	tiles.sort_by_key(|tile| (tile.x, tile.z));
	Ok(WorldMapManifest {
		pack_key: pack.pack_key,
		atlas: pack.atlas,
		atlas_width: pack.atlas_width,
		atlas_height: pack.atlas_height,
		tables_version: tables_version(&folder_dir.join("structure")).await,
		sections,
		tiles,
	})
}

/// When a region was last drawn, in Unix milliseconds. Its detail tiles are named by it.
pub async fn region_version(folder_dir: &Path, region: (i32, i32)) -> Result<i64> {
	let stamp: RegionStamp = read_json(&folder_dir.join("regions").join(format!("r.{}.{}.json", region.0, region.1))).await?;
	Ok(stamp.rendered_at)
}

/// Stamps the block and biome tables by size and modification time, which costs two metadata reads rather than a hash.
pub(crate) async fn tables_version(structure: &Path) -> String {
	let blocks = table_stamp(&structure.join("blocks.json")).await;
	let biomes = table_stamp(&structure.join("biomes.json")).await;
	format!("{blocks}.{biomes}")
}

async fn table_stamp(path: &Path) -> String {
	match tokio::fs::metadata(path).await {
		Ok(metadata) => {
			let modified = metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok()).map_or(0, |elapsed| elapsed.as_nanos());
			format!("{}-{modified}", metadata.len())
		}
		Err(_) => "0".to_owned(),
	}
}

/// Turns a client path into a path inside the map folder. Only plain names are accepted, so nothing can climb out.
pub fn file_path(map_dir: &Path, relative: &str) -> Result<PathBuf> {
	let path = Path::new(relative);
	let plain = !relative.is_empty() && path.components().all(|component| matches!(component, Component::Normal(_)));
	if !plain {
		return Err(CommonError::Invalid(format!("{relative} is not a map file")));
	}
	Ok(map_dir.join(path))
}

async fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
	let bytes = tokio::fs::read(path)
		.await
		.map_err(|_| CommonError::NotFound(format!("{} is not rendered yet", path.display())))?;
	Ok(serde_json::from_slice(&bytes)?)
}
