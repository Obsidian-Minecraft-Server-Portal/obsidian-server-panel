//! Turns the faces of model elements into quads in block space, applying element and blockstate rotations.
//! Vertex order is counter-clockwise seen from outside the block, so faces wind the way renderers expect.

use std::collections::BTreeMap;

use crate::models::{Element, ElementRotation};

pub const MISSING_TEXTURE: &str = "minecraft:missing";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
	Down,
	Up,
	North,
	South,
	West,
	East,
}

impl Direction {
	pub const ALL: [Direction; 6] = [Self::Down, Self::Up, Self::North, Self::South, Self::West, Self::East];

	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"down" => Some(Self::Down),
			"up" => Some(Self::Up),
			"north" => Some(Self::North),
			"south" => Some(Self::South),
			"west" => Some(Self::West),
			"east" => Some(Self::East),
			_ => None,
		}
	}

	pub fn index(self) -> u8 {
		self as u8
	}

	pub fn from_index(index: u8) -> Option<Self> {
		Self::ALL.get(usize::from(index)).copied()
	}

	pub fn normal(self) -> [f32; 3] {
		match self {
			Self::Down => [0.0, -1.0, 0.0],
			Self::Up => [0.0, 1.0, 0.0],
			Self::North => [0.0, 0.0, -1.0],
			Self::South => [0.0, 0.0, 1.0],
			Self::West => [-1.0, 0.0, 0.0],
			Self::East => [1.0, 0.0, 0.0],
		}
	}

	pub fn opposite(self) -> Self {
		match self {
			Self::Down => Self::Up,
			Self::Up => Self::Down,
			Self::North => Self::South,
			Self::South => Self::North,
			Self::West => Self::East,
			Self::East => Self::West,
		}
	}

	/// Offset to the neighbouring block this face points at.
	pub fn offset(self) -> [i32; 3] {
		self.normal().map(|component| component as i32)
	}
}

/// Which tint a face takes from the biome colormaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TintKind {
	Grass,
	Foliage,
	Water,
}

pub fn tint_kind_for_block(name: &str) -> Option<TintKind> {
	if name == "minecraft:water" {
		Some(TintKind::Water)
	} else if name.ends_with("_leaves") {
		Some(TintKind::Foliage)
	} else if ["grass_block", "grass", "fern", "tall_grass", "short_grass"].iter().any(|suffix| name.ends_with(suffix)) {
		Some(TintKind::Grass)
	} else {
		None
	}
}

/// A rectangle in block space (0..16 per axis) with texture coordinates in 0..16 texture pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Quad {
	pub corners: [[f32; 3]; 4],
	pub uvs: [[f32; 2]; 4],
	pub texture: String,
	pub direction: Direction,
	pub cullface: Option<Direction>,
	pub tint: Option<TintKind>,
}

/// Quads for every face of one element, before any blockstate rotation.
pub fn element_quads(element: &Element, textures: &BTreeMap<String, String>, tint: Option<TintKind>) -> Vec<Quad> {
	let mut quads = Vec::new();
	for (name, face) in &element.faces {
		let Some(direction) = Direction::from_name(name) else { continue };
		let Some(mut corners) = face_corners(direction, element.from, element.to) else { continue };
		let default = default_rect(direction, element.from, element.to);
		let rect = face.uv.unwrap_or(default);
		let mut uvs = [[0.0; 2]; 4];
		for (slot, corner) in uvs.iter_mut().zip(corners) {
			let (du, dv) = default_uv(direction, corner);
			let u = rect[0] + (du - default[0]) / (default[2] - default[0]) * (rect[2] - rect[0]);
			let v = rect[1] + (dv - default[1]) / (default[3] - default[1]) * (rect[3] - rect[1]);
			*slot = rotate_uv([u, v], quarter_turns(face.rotation));
		}
		if let Some(rotation) = &element.rotation {
			corners = corners.map(|corner| rotate_element(corner, rotation));
		}
		quads.push(Quad {
			corners,
			uvs,
			texture: resolve_texture(&face.texture, textures),
			direction,
			cullface: face.cullface.as_deref().and_then(Direction::from_name),
			tint: face.tintindex.and(tint),
		});
	}
	quads
}

/// Applies a blockstate's `x` and `y` rotations, and keeps textures upright when `uvlock` is set.
pub fn transform_quad(quad: &Quad, x: i32, y: i32, uvlock: bool) -> Quad {
	let rotate_point = |point: [f32; 3]| {
		let centred = point.map(|component| component - 8.0);
		rotate_by_blockstate(centred, x, y).map(|component| component + 8.0)
	};
	let rotate_direction = |direction: Direction| nearest_direction(rotate_by_blockstate(direction.normal(), x, y));
	let uv_turns = if uvlock && y != 0 { quarter_turns(360 - y) } else { 0 };
	Quad {
		corners: quad.corners.map(rotate_point),
		uvs: quad.uvs.map(|uv| rotate_uv(uv, uv_turns)),
		texture: quad.texture.clone(),
		direction: rotate_direction(quad.direction),
		cullface: quad.cullface.map(rotate_direction),
		tint: quad.tint,
	}
}

fn face_corners(direction: Direction, from: [f32; 3], to: [f32; 3]) -> Option<[[f32; 3]; 4]> {
	let (plane, a, b, value) = match direction {
		Direction::Down => (1, 0, 2, from[1]),
		Direction::Up => (1, 0, 2, to[1]),
		Direction::North => (2, 0, 1, from[2]),
		Direction::South => (2, 0, 1, to[2]),
		Direction::West => (0, 2, 1, from[0]),
		Direction::East => (0, 2, 1, to[0]),
	};
	if from[a] == to[a] || from[b] == to[b] {
		return None;
	}
	let corner = |along_a: f32, along_b: f32| {
		let mut point = [0.0; 3];
		point[plane] = value;
		point[a] = along_a;
		point[b] = along_b;
		point
	};
	let corners = [
		corner(from[a], from[b]),
		corner(to[a], from[b]),
		corner(to[a], to[b]),
		corner(from[a], to[b]),
	];
	let mut axis_a = [0.0; 3];
	axis_a[a] = 1.0;
	let mut axis_b = [0.0; 3];
	axis_b[b] = 1.0;
	let facing = dot(cross(axis_a, axis_b), direction.normal());
	Some(if facing < 0.0 { [corners[3], corners[2], corners[1], corners[0]] } else { corners })
}

/// The face's default texture rectangle, as Minecraft maps a box onto a texture when no `uv` is given.
fn default_rect(direction: Direction, from: [f32; 3], to: [f32; 3]) -> [f32; 4] {
	match direction {
		Direction::Down => [from[0], 16.0 - to[2], to[0], 16.0 - from[2]],
		Direction::Up => [from[0], from[2], to[0], to[2]],
		Direction::North => [16.0 - to[0], 16.0 - to[1], 16.0 - from[0], 16.0 - from[1]],
		Direction::South => [from[0], 16.0 - to[1], to[0], 16.0 - from[1]],
		Direction::West => [from[2], 16.0 - to[1], to[2], 16.0 - from[1]],
		Direction::East => [16.0 - to[2], 16.0 - to[1], 16.0 - from[2], 16.0 - from[1]],
	}
}

fn default_uv(direction: Direction, point: [f32; 3]) -> (f32, f32) {
	match direction {
		Direction::Down => (point[0], 16.0 - point[2]),
		Direction::Up => (point[0], point[2]),
		Direction::North => (16.0 - point[0], 16.0 - point[1]),
		Direction::South => (point[0], 16.0 - point[1]),
		Direction::West => (point[2], 16.0 - point[1]),
		Direction::East => (16.0 - point[2], 16.0 - point[1]),
	}
}

/// Texture references without a namespace belong to `minecraft`, as they do in the game.
fn resolve_texture(reference: &str, textures: &BTreeMap<String, String>) -> String {
	let resolved = match reference.strip_prefix('#') {
		Some(variable) => textures.get(variable).map_or(MISSING_TEXTURE, String::as_str),
		None => reference,
	};
	if resolved.contains(':') {
		resolved.to_owned()
	} else {
		format!("minecraft:{resolved}")
	}
}

fn quarter_turns(degrees: i32) -> i32 {
	degrees.rem_euclid(360) / 90
}

/// Rotates a texture coordinate clockwise about the texture centre.
fn rotate_uv(uv: [f32; 2], turns: i32) -> [f32; 2] {
	(0..turns).fold(uv, |[u, v], _| [16.0 - v, u])
}

fn rotate_element(point: [f32; 3], rotation: &ElementRotation) -> [f32; 3] {
	let axis = match rotation.axis.as_str() {
		"x" => 0,
		"y" => 1,
		_ => 2,
	};
	let relative = [0, 1, 2].map(|index| point[index] - rotation.origin[index]);
	let mut rotated = rotate_about(relative, axis, rotation.angle);
	if rotation.rescale {
		let scale = 1.0 / rotation.angle.to_radians().cos().abs().max(f32::EPSILON);
		for (index, component) in rotated.iter_mut().enumerate() {
			if index != axis {
				*component *= scale;
			}
		}
	}
	[0, 1, 2].map(|index| rotated[index] + rotation.origin[index])
}

/// Blockstate rotations turn the block clockwise (viewed from above for `y`), which is a right-handed turn by `-angle`.
fn rotate_by_blockstate(vector: [f32; 3], x: i32, y: i32) -> [f32; 3] {
	let after_x = rotate_about(vector, 0, -(x as f32));
	rotate_about(after_x, 1, -(y as f32))
}

fn rotate_about(vector: [f32; 3], axis: usize, degrees: f32) -> [f32; 3] {
	let (sin, cos) = degrees.to_radians().sin_cos();
	let [x, y, z] = vector;
	match axis {
		0 => [x, y * cos - z * sin, y * sin + z * cos],
		1 => [x * cos + z * sin, y, -x * sin + z * cos],
		_ => [x * cos - y * sin, x * sin + y * cos, z],
	}
}

fn nearest_direction(vector: [f32; 3]) -> Direction {
	let (axis, value) = (0..3)
		.map(|axis| (axis, vector[axis]))
		.max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
		.unwrap_or((1, 1.0));
	match (axis, value >= 0.0) {
		(0, true) => Direction::East,
		(0, false) => Direction::West,
		(1, true) => Direction::Up,
		(1, false) => Direction::Down,
		(_, true) => Direction::South,
		(_, false) => Direction::North,
	}
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
	[a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
	a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::Face;

	fn face(texture: &str) -> Face {
		Face {
			uv: None,
			texture: texture.to_owned(),
			cullface: None,
			rotation: 0,
			tintindex: None,
		}
	}

	fn cube(faces: &[(&str, Face)]) -> Element {
		Element {
			from: [0.0; 3],
			to: [16.0; 3],
			rotation: None,
			faces: faces.iter().map(|(name, face)| ((*name).to_owned(), face.clone())).collect(),
		}
	}

	fn outward(quad: &Quad) -> bool {
		let [c0, c1, c2, _] = quad.corners;
		let edge_a = [0, 1, 2].map(|index| c1[index] - c0[index]);
		let edge_b = [0, 1, 2].map(|index| c2[index] - c0[index]);
		dot(cross(edge_a, edge_b), quad.direction.normal()) > 0.0
	}

	#[test]
	fn every_face_of_a_cube_winds_outward_with_full_uvs() {
		let textures = BTreeMap::from([("all".to_owned(), "minecraft:block/stone".to_owned())]);
		let faces: Vec<(&str, Face)> = ["down", "up", "north", "south", "west", "east"].iter().map(|name| (*name, face("#all"))).collect();
		let quads = element_quads(&cube(&faces), &textures, None);
		assert_eq!(quads.len(), 6);
		for quad in &quads {
			assert!(outward(quad), "{:?} winds inward", quad.direction);
			let us: Vec<f32> = quad.uvs.iter().map(|uv| uv[0]).collect();
			assert!(us.contains(&0.0) && us.contains(&16.0));
			assert_eq!(quad.texture, "minecraft:block/stone");
		}
	}

	#[test]
	fn custom_uv_rectangles_and_face_rotation_map_textures() {
		let textures = BTreeMap::from([("all".to_owned(), "minecraft:block/stone".to_owned())]);
		let mut up = face("#all");
		up.uv = Some([0.0, 0.0, 8.0, 8.0]);
		let plain = element_quads(&cube(&[("up", up.clone())]), &textures, None);
		let max_u = plain[0].uvs.iter().map(|uv| uv[0]).fold(f32::MIN, f32::max);
		assert_eq!(max_u, 8.0);
		up.rotation = 90;
		let turned = element_quads(&cube(&[("up", up)]), &textures, None);
		let turned_max_u = turned[0].uvs.iter().map(|uv| uv[0]).fold(f32::MIN, f32::max);
		let turned_max_v = turned[0].uvs.iter().map(|uv| uv[1]).fold(f32::MIN, f32::max);
		assert_eq!((turned_max_u, turned_max_v), (16.0, 8.0));
	}

	#[test]
	fn blockstate_y_turns_north_into_east_and_moves_the_culling_side() {
		let textures = BTreeMap::new();
		let mut north = face("minecraft:block/furnace_front");
		north.cullface = Some("north".to_owned());
		let quads = element_quads(&cube(&[("north", north)]), &textures, None);
		let turned = transform_quad(&quads[0], 0, 90, false);
		assert_eq!(turned.direction, Direction::East);
		assert_eq!(turned.cullface, Some(Direction::East));
	}

	#[test]
	fn uvlock_keeps_the_texture_upright_after_a_y_turn() {
		let textures = BTreeMap::new();
		let mut north = face("minecraft:block/log");
		north.uv = Some([0.0, 0.0, 16.0, 16.0]);
		let quads = element_quads(&cube(&[("north", north)]), &textures, None);
		let locked = transform_quad(&quads[0], 0, 90, true);
		let before = quads[0].uvs[0];
		assert_eq!(locked.uvs[0], rotate_uv(before, 3));
	}

	#[test]
	fn element_rotation_turns_points_about_their_origin() {
		let rotation = ElementRotation {
			origin: [8.0, 8.0, 8.0],
			axis: "y".to_owned(),
			angle: 90.0,
			rescale: false,
		};
		let turned = rotate_element([10.0, 8.0, 8.0], &rotation);
		assert!((turned[0] - 8.0).abs() < 1e-5 && (turned[2] - 6.0).abs() < 1e-5, "{turned:?}");
	}
}
