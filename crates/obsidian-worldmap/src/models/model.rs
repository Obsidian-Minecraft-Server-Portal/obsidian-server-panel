use std::collections::BTreeMap;

use serde::Deserialize;

/// `assets/<namespace>/models/<path>.json`. Missing fields are inherited from the parent model.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelFile {
	#[serde(default)]
	pub parent: Option<String>,
	#[serde(default)]
	pub textures: BTreeMap<String, TextureValue>,
	#[serde(default)]
	pub elements: Option<Vec<Element>>,
	#[serde(default)]
	pub ambientocclusion: Option<bool>,
}

/// A texture entry is a sprite id, or an object naming one. `force_translucent` puts that sprite on the translucent layer.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TextureValue {
	Id(String),
	Sprite {
		sprite: String,
		#[serde(default)]
		force_translucent: bool,
	},
}

impl TextureValue {
	pub fn id(&self) -> &str {
		match self {
			Self::Id(id) | Self::Sprite { sprite: id, .. } => id.as_str(),
		}
	}

	pub fn force_translucent(&self) -> bool {
		matches!(self, Self::Sprite { force_translucent: true, .. })
	}
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
	/// `false` draws the element's faces at full brightness, with no directional shading.
	#[serde(default)]
	pub shade: Option<bool>,
	/// The direction whose shade every face of the element takes, in place of the face's own direction.
	#[serde(default)]
	pub shade_direction_override: Option<String>,
}

/// A box's turn about its origin: one angle about an axis, or Euler angles in degrees applied about x, then y, then z.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawRotation")]
pub struct ElementRotation {
	pub origin: [f32; 3],
	pub value: RotationValue,
	pub rescale: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RotationValue {
	SingleAxis { axis: usize, angle: f32 },
	Euler([f32; 3]),
}

#[derive(Deserialize)]
struct RawRotation {
	origin: [f32; 3],
	#[serde(default)]
	axis: Option<String>,
	#[serde(default)]
	angle: f32,
	#[serde(default)]
	x: f32,
	#[serde(default)]
	y: f32,
	#[serde(default)]
	z: f32,
	#[serde(default)]
	rescale: bool,
}

impl From<RawRotation> for ElementRotation {
	fn from(raw: RawRotation) -> Self {
		let value = match raw.axis.as_deref() {
			Some(axis) => RotationValue::SingleAxis {
				axis: match axis {
					"x" => 0,
					"y" => 1,
					_ => 2,
				},
				angle: raw.angle,
			},
			None => RotationValue::Euler([raw.x, raw.y, raw.z]),
		};
		Self {
			origin: raw.origin,
			value,
			rescale: raw.rescale,
		}
	}
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
