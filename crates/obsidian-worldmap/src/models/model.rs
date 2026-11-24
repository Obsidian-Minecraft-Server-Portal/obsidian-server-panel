use std::collections::BTreeMap;

use serde::Deserialize;

/// `assets/<namespace>/models/<path>.json`. Missing fields are inherited from the parent model.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelFile {
	#[serde(default)]
	pub parent: Option<String>,
	#[serde(default)]
	pub textures: BTreeMap<String, String>,
	#[serde(default)]
	pub elements: Option<Vec<Element>>,
}

/// A box in 0..16 block units with per-face texture data.
#[derive(Debug, Clone, Deserialize)]
pub struct Element {
	pub from: [f32; 3],
	pub to: [f32; 3],
	#[serde(default)]
	pub rotation: Option<ElementRotation>,
	#[serde(default)]
	pub faces: BTreeMap<String, Face>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ElementRotation {
	pub origin: [f32; 3],
	pub axis: String,
	pub angle: f32,
	#[serde(default)]
	pub rescale: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Face {
	/// `[x1, y1, x2, y2]` in 0..16 texture units; omitted means the box's own extent is used.
	#[serde(default)]
	pub uv: Option<[f32; 4]>,
	pub texture: String,
	#[serde(default)]
	pub cullface: Option<String>,
	#[serde(default)]
	pub rotation: i32,
	#[serde(default)]
	pub tintindex: Option<i32>,
}
