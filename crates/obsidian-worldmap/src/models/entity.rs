//! Cubes of vanilla entity models, baked to quads the way the game bakes them: each polygon's vertices take the texels of its
//! rectangle in the game's order, and a placement moves the model into its block.

use super::geometry::{Direction, Quad, nearest_direction, rotate_about};

/// Entity textures are 64 texels square, and block UVs span 16 units across a texture.
const TEXEL_UNITS: f32 = 16.0 / 64.0;

/// A cube of an entity model, in model units (16 to a block): its corner and size, the texel where its texture layout starts, and a
/// face to leave out.
pub struct Cube {
	pub from: [f32; 3],
	pub size: [f32; 3],
	pub texture_offset: (u32, u32),
	pub hidden: Option<Direction>,
}

/// Where a model stands in its block. A point moves by `translate`, turns about `pivot` by each turn in order, then moves by `after`.
pub struct Placement {
	pub pivot: [f32; 3],
	pub translate: [f32; 3],
	/// An axis (0 for x, 1 for y and 2 for z) and a right-handed angle in degrees.
	pub turns: Vec<(usize, f32)>,
	pub after: [f32; 3],
}

impl Placement {
	fn point(&self, point: [f32; 3]) -> [f32; 3] {
		let relative = [0, 1, 2].map(|axis| point[axis] + self.translate[axis] - self.pivot[axis]);
		let turned = self.turns.iter().fold(relative, |turned, &(axis, degrees)| rotate_about(turned, axis, degrees));
		[0, 1, 2].map(|axis| turned[axis] + self.pivot[axis] + self.after[axis])
	}

	fn direction(&self, direction: Direction) -> Direction {
		let normal = self.turns.iter().fold(direction.normal(), |normal, &(axis, degrees)| rotate_about(normal, axis, degrees));
		nearest_direction(normal)
	}
}

/// The quads of a cube, in block units, all drawn from one texture.
pub fn cube_quads(cube: &Cube, placement: &Placement, texture: &str) -> Vec<Quad> {
	let [x, y, z] = cube.from;
	let [w, h, d] = cube.size;
	let (x2, y2, z2) = (x + w, y + h, z + d);
	let (tu, tv) = (cube.texture_offset.0 as f32, cube.texture_offset.1 as f32);
	let (u1, u2, u3) = (tu + d, tu + d + w, tu + d + 2.0 * w);
	let (u4, u5) = (tu + 2.0 * d + w, tu + 2.0 * d + 2.0 * w);
	let (v0, v1, v2) = (tv, tv + d, tv + d + h);
	let at = |max_x: bool, max_y: bool, max_z: bool| [if max_x { x2 } else { x }, if max_y { y2 } else { y }, if max_z { z2 } else { z }];
	// Each polygon: its direction, its vertices in the game's order, and the texel rectangle `[u1, v1, u2, v2]` they lie on.
	let polygons = [
		(Direction::Down, [at(true, false, true), at(false, false, true), at(false, false, false), at(true, false, false)], [u1, v0, u2, v1]),
		(Direction::Up, [at(true, true, false), at(false, true, false), at(false, true, true), at(true, true, true)], [u2, v1, u3, v0]),
		(Direction::West, [at(false, false, false), at(false, false, true), at(false, true, true), at(false, true, false)], [tu, v1, u1, v2]),
		(Direction::North, [at(true, false, false), at(false, false, false), at(false, true, false), at(true, true, false)], [u1, v1, u2, v2]),
		(Direction::East, [at(true, false, true), at(true, false, false), at(true, true, false), at(true, true, true)], [u2, v1, u4, v2]),
		(Direction::South, [at(false, false, true), at(true, false, true), at(true, true, true), at(false, true, true)], [u4, v1, u5, v2]),
	];
	polygons
		.into_iter()
		.filter(|(direction, _, _)| Some(*direction) != cube.hidden)
		.map(|(direction, vertices, rect)| {
			// Vertices take the texel corners (u2, v1), (u1, v1), (u1, v2) and (u2, v2) in turn.
			let texels = [[rect[2], rect[1]], [rect[0], rect[1]], [rect[0], rect[3]], [rect[2], rect[3]]];
			let corners = vertices.map(|point| placement.point(point));
			let uvs = texels.map(|[u, v]| [u * TEXEL_UNITS, v * TEXEL_UNITS]);
			let normal = placement.direction(direction);
			let (corners, uvs) = wound_outward(corners, uvs, normal);
			Quad {
				corners,
				uvs,
				texture: texture.to_owned(),
				direction: normal,
				cullface: None,
				shade: Some(normal),
				tint: None,
				ambient_occlusion: false,
				fluid: false,
			}
		})
		.collect()
}

/// Reverses a quad whose corners wind inward, so every face faces out of the block.
fn wound_outward(corners: [[f32; 3]; 4], uvs: [[f32; 2]; 4], normal: Direction) -> ([[f32; 3]; 4], [[f32; 2]; 4]) {
	let edge_a = [0, 1, 2].map(|axis| corners[1][axis] - corners[0][axis]);
	let edge_b = [0, 1, 2].map(|axis| corners[2][axis] - corners[0][axis]);
	let cross = [edge_a[1] * edge_b[2] - edge_a[2] * edge_b[1], edge_a[2] * edge_b[0] - edge_a[0] * edge_b[2], edge_a[0] * edge_b[1] - edge_a[1] * edge_b[0]];
	let facing: f32 = cross.iter().zip(normal.normal()).map(|(a, b)| a * b).sum();
	if facing >= 0.0 {
		(corners, uvs)
	} else {
		([corners[3], corners[2], corners[1], corners[0]], [uvs[3], uvs[2], uvs[1], uvs[0]])
	}
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
