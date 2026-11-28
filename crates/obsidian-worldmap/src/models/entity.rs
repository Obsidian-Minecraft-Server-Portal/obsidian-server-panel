//! Cuboids of entity models, with each face laid out on a 64-texel texture the way the game lays out a box's cubes.

use std::collections::BTreeMap;

use super::{Element, Face};

/// The side of an entity texture in texels. Block UVs span 16 units across a texture, so each unit is four texels.
pub const TEXTURE_TEXELS: f32 = 64.0;

/// One box of an entity model, in block units: its corner, its size, the texel its texture layout starts at, and a face to leave out.
pub struct Cuboid {
	pub from: [f32; 3],
	pub size: [f32; 3],
	pub texture_offset: (u32, u32),
	pub hidden: Option<&'static str>,
}

impl Cuboid {
	/// The element for this box, with every face drawn from the texture variable `variable`.
	pub fn element(&self, variable: &str) -> Element {
		let scale = 16.0 / TEXTURE_TEXELS;
		let faces: BTreeMap<String, Face> = face_layout(self.texture_offset, self.size)
			.into_iter()
			.filter(|(name, _)| Some(*name) != self.hidden)
			.map(|(name, rect)| {
				let face = Face {
					uv: Some(rect.map(|texel| texel * scale)),
					texture: format!("#{variable}"),
					cullface: None,
					rotation: 0,
					tintindex: None,
				};
				(name.to_owned(), face)
			})
			.collect();
		Element {
			from: self.from,
			to: std::array::from_fn(|axis| self.from[axis] + self.size[axis]),
			rotation: None,
			faces,
			shade: None,
			shade_direction_override: None,
		}
	}
}

/// The texel rectangle of each face: the top and bottom in the first row, then the west, north, east and south.
fn face_layout((u, v): (u32, u32), [width, height, depth]: [f32; 3]) -> [(&'static str, [f32; 4]); 6] {
	let (u, v) = (u as f32, v as f32);
	[
		("up", [u + depth, v, u + depth + width, v + depth]),
		("down", [u + depth + width, v, u + depth + 2.0 * width, v + depth]),
		("west", [u, v + depth, u + depth, v + depth + height]),
		("north", [u + depth, v + depth, u + depth + width, v + depth + height]),
		("east", [u + depth + width, v + depth, u + 2.0 * depth + width, v + depth + height]),
		("south", [u + 2.0 * depth + width, v + depth, u + 2.0 * depth + 2.0 * width, v + depth + height]),
	]
}

/// The `toYRot` of a horizontal facing, which is how far the game turns an entity model from south.
pub fn facing_turn(facing: Option<&str>) -> i32 {
	match facing {
		Some("west") => 90,
		Some("north") => 180,
		Some("east") => 270,
		_ => 0,
	}
}
