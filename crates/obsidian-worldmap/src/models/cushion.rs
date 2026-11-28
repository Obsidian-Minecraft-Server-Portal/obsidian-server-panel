//! Cushions are entities that lie on the block under them: a 16 by 4 by 16 slab, turned to the facing it was placed with. The overlay
//! gives each one the voxel that holds its top, and the height of its bottom above that voxel's floor.

use super::entity::{Cube, Placement, cube_quads, facing_turn};
use super::geometry::Quad;
use crate::chunk::BlockState;

/// The block state a cushion takes in a structure. The entity overlay writes it into the voxel that holds the cushion's top.
pub const STATE: &str = "minecraft:cushion";

const COLOURS: [&str; 16] = [
	"white",
	"orange",
	"magenta",
	"light_blue",
	"yellow",
	"lime",
	"pink",
	"gray",
	"light_gray",
	"cyan",
	"purple",
	"blue",
	"brown",
	"green",
	"red",
	"black",
];

/// The cushion textures, one for each dye colour.
pub fn textures() -> Vec<String> {
	COLOURS.iter().map(|colour| format!("minecraft:entity/cushion/{colour}_cushion")).collect()
}

/// The quads of a cushion state, or `None` when the state is not a cushion.
pub fn cushion_quads(state: &BlockState) -> Option<Vec<Quad>> {
	if state.name != STATE {
		return None;
	}
	let colour = state.properties.get("color").map_or("white", String::as_str);
	let offset = state.properties.get("offset").and_then(|offset| offset.parse::<f32>().ok()).unwrap_or(0.0);
	let facing = facing_turn(state.properties.get("facing").map(String::as_str));
	let cube = Cube {
		from: [-8.0, 0.0, -8.0],
		size: [16.0, 4.0, 16.0],
		texture_offset: (0, 0),
		hidden: None,
	};
	// The game moves the model down a quarter block, turns it about the x axis, then turns it by 180 degrees less its facing. The
	// model's origin is the middle of its block's floor, raised by `offset` sixteenths.
	let placement = Placement {
		pivot: [0.0; 3],
		translate: [0.0, -4.0, 0.0],
		turns: vec![(0, 180.0), (1, (180 - facing) as f32)],
		after: [8.0, offset, 8.0],
	};
	Some(cube_quads(&cube, &placement, &format!("minecraft:entity/cushion/{colour}_cushion")))
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use super::*;
	use crate::models::geometry::Direction;

	fn cushion(facing: &str, offset: &str) -> BlockState {
		BlockState {
			name: STATE.into(),
			properties: BTreeMap::from([
				("color".to_owned(), "red".to_owned()),
				("facing".to_owned(), facing.to_owned()),
				("offset".to_owned(), offset.to_owned()),
			]),
		}
	}

	#[test]
	fn a_cushion_is_a_slab_on_the_floor_of_its_block() {
		let quads = cushion_quads(&cushion("south", "0")).unwrap();
		assert_eq!(quads.len(), 6);
		let within = |value: f32, low: f32, high: f32| value >= low - 1e-3 && value <= high + 1e-3;
		assert!(quads
			.iter()
			.flat_map(|quad| quad.corners)
			.all(|corner| within(corner[0], 0.0, 16.0) && within(corner[1], 0.0, 4.0) && within(corner[2], 0.0, 16.0)));
		assert!(quads.iter().all(|quad| quad.texture == "minecraft:entity/cushion/red_cushion"));
	}

	#[test]
	fn the_offset_lifts_the_slab_within_its_voxel() {
		let quads = cushion_quads(&cushion("south", "-1")).unwrap();
		assert!(quads.iter().flat_map(|quad| quad.corners).all(|corner| (-1.0 - 1e-3..=3.0 + 1e-3).contains(&corner[1])));
	}

	#[test]
	fn the_slab_has_its_floor_face_down_on_the_floor() {
		let quads = cushion_quads(&cushion("north", "0")).unwrap();
		let at = |height: f32| quads.iter().filter(move |quad| quad.corners.iter().all(|corner| (corner[1] - height).abs() < 1e-3));
		assert!(at(0.0).any(|quad| quad.direction == Direction::Down));
		assert!(at(4.0).any(|quad| quad.direction == Direction::Up));
	}

	#[test]
	fn other_states_are_not_cushions() {
		assert!(cushion_quads(&BlockState {
			name: "minecraft:stone".into(),
			properties: BTreeMap::new(),
		})
		.is_none());
	}
}
