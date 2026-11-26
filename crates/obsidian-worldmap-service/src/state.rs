//! Which region files were rendered, from which pack, and when. The viewer does not read this; it only decides what to render.

use std::collections::BTreeMap;
use std::path::Path;

use obsidian_common::Result;
use serde::{Deserialize, Serialize};

const STATE_FILE: &str = "state.json";

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RenderState {
	pub pack_key: Option<String>,
	pub last_full_pass: Option<i64>,
	/// Keyed by dimension folder and region, such as `minecraft_overworld/r.0.0`.
	pub regions: BTreeMap<String, RegionState>,
	/// Regions whose last render failed, with the modification time it failed on. They are retried only once the file changes.
	pub failed: BTreeMap<String, i64>,
	/// Unix milliseconds of the last render that drew a region, so a viewer knows when to fetch the map again.
	pub revision: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionState {
	/// Modification time of the region file that was rendered, in seconds since the epoch.
	pub source_modified: i64,
	pub rendered_at: i64,
}

/// A damaged state file is treated as missing, which makes the next render a full one.
pub async fn load(map_dir: &Path) -> RenderState {
	match tokio::fs::read(map_dir.join(STATE_FILE)).await {
		Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
		Err(_) => RenderState::default(),
	}
}

pub async fn save(map_dir: &Path, state: &RenderState) -> Result<()> {
	tokio::fs::create_dir_all(map_dir).await?;
	let temp = map_dir.join("state.json.tmp");
	tokio::fs::write(&temp, serde_json::to_vec(state)?).await?;
	tokio::fs::rename(&temp, map_dir.join(STATE_FILE)).await?;
	Ok(())
}
