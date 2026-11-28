//! Cushions are entities that lie on the floor of the block they occupy: a 16 by 4 by 16 slab, turned to face the way it was placed.

use std::collections::BTreeMap;

use super::entity::{Cuboid, facing_turn};
use super::geometry::{Quad, element_quads, transform_quad};
use crate::chunk::BlockState;

/// The block state a cushion takes in a structure. The structure's entity overlay writes it into the voxel the cushion occupies.
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
	let variables = BTreeMap::from([("cushion".to_owned(), format!("minecraft:entity/cushion/{colour}_cushion"))]);
	let element = Cuboid {
		from: [0.0, 12.0, 0.0],
		size: [16.0, 4.0, 16.0],
		texture_offset: (0, 0),
		hidden: None,
	}
	.element("cushion");
	// The game flips the model about the x axis and then turns it by 180 degrees less its facing. The box starts 12 units higher,
	// so that the flip lands it on the floor of the block.
	let turn = (facing_turn(state.properties.get("facing").map(String::as_str)) + 180).rem_euclid(360);
	Some(element_quads(&element, &variables, None, false).iter().map(|quad| transform_quad(quad, 180, turn, false)).collect())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::geometry::Direction;

	fn cushion(facing: &str) -> BlockState {
		BlockState {
			name: STATE.into(),
			properties: BTreeMap::from([("color".to_owned(), "red".to_owned()), ("facing".to_owned(), facing.to_owned())]),
		}
	}

	#[test]
	fn a_cushion_is_a_slab_in_the_bottom_four_units_of_its_block() {
		let quads = cushion_quads(&cushion("south")).unwrap();
		assert_eq!(quads.len(), 6);
		let within = |value: f32, high: f32| value >= -1e-3 && value <= high + 1e-3;
		assert!(quads
			.iter()
			.flat_map(|quad| quad.corners)
			.all(|corner| within(corner[0], 16.0) && within(corner[1], 4.0) && within(corner[2], 16.0)));
		assert!(quads.iter().all(|quad| quad.texture == "minecraft:entity/cushion/red_cushion"));
	}

	#[test]
	fn the_slab_rests_on_the_floor_with_its_faces_turned_outward() {
		let quads = cushion_quads(&cushion("north")).unwrap();
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
