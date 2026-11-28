//! Chests are block entities, so their block states draw nothing. Each chest is built from the cubes of vanilla's chest model, turned
//! to its facing. A double chest draws one half per block and leaves out the face between the halves.

use super::entity::{Cube, Placement, cube_quads, facing_turn};
use super::geometry::{Direction, Quad};
use crate::chunk::BlockState;

/// The chest textures, each 64 texels square. The atlas packs each whole.
pub const TEXTURES: [&str; 19] = [
	"minecraft:entity/chest/normal",
	"minecraft:entity/chest/normal_left",
	"minecraft:entity/chest/normal_right",
	"minecraft:entity/chest/trapped",
	"minecraft:entity/chest/trapped_left",
	"minecraft:entity/chest/trapped_right",
	"minecraft:entity/chest/ender",
	"minecraft:entity/chest/copper",
	"minecraft:entity/chest/copper_left",
	"minecraft:entity/chest/copper_right",
	"minecraft:entity/chest/copper_exposed",
	"minecraft:entity/chest/copper_exposed_left",
	"minecraft:entity/chest/copper_exposed_right",
	"minecraft:entity/chest/copper_weathered",
	"minecraft:entity/chest/copper_weathered_left",
	"minecraft:entity/chest/copper_weathered_right",
	"minecraft:entity/chest/copper_oxidized",
	"minecraft:entity/chest/copper_oxidized_left",
	"minecraft:entity/chest/copper_oxidized_right",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Half {
	Single,
	Left,
	Right,
}

/// The quads of a chest state, or `None` when the state is not a chest.
pub fn chest_quads(state: &BlockState) -> Option<Vec<Quad>> {
	let family = family(&state.name)?;
	let half = match state.properties.get("type").map(String::as_str) {
		_ if family == "ender" => Half::Single,
		Some("left") => Half::Left,
		Some("right") => Half::Right,
		_ => Half::Single,
	};
	let suffix = match half {
		Half::Single => "",
		Half::Left => "_left",
		Half::Right => "_right",
	};
	let texture = format!("minecraft:entity/chest/{family}{suffix}");
	let placement = Placement {
		pivot: [8.0, 0.0, 8.0],
		translate: [0.0; 3],
		turns: vec![(1, -(facing_turn(state.properties.get("facing").map(String::as_str)) as f32))],
		after: [0.0; 3],
	};
	Some(cubes(half).iter().flat_map(|cube| cube_quads(cube, &placement, &texture)).collect())
}

/// The texture family of a chest block: its name without any `waxed_` prefix, mapped to the names of its textures.
fn family(name: &str) -> Option<&'static str> {
	Some(match name.strip_prefix("minecraft:")?.trim_start_matches("waxed_") {
		"chest" => "normal",
		"trapped_chest" => "trapped",
		"ender_chest" => "ender",
		"copper_chest" => "copper",
		"exposed_copper_chest" => "copper_exposed",
		"weathered_copper_chest" => "copper_weathered",
		"oxidized_copper_chest" => "copper_oxidized",
		_ => return None,
	})
}

/// The bottom, the lid and the lock of a chest half, in the game's model. A double chest's halves are one block wide each, and the
/// face between them is left out.
fn cubes(half: Half) -> [Cube; 3] {
	let (hidden, body_x, body_width, lock_x, lock_width) = match half {
		Half::Single => (None, 1.0, 14.0, 7.0, 2.0),
		Half::Left => (Some(Direction::West), 0.0, 15.0, 0.0, 1.0),
		Half::Right => (Some(Direction::East), 1.0, 15.0, 15.0, 1.0),
	};
	[
		Cube {
			from: [body_x, 0.0, 1.0],
			size: [body_width, 10.0, 14.0],
			texture_offset: (0, 19),
			hidden,
		},
		Cube {
			from: [body_x, 9.0, 1.0],
			size: [body_width, 5.0, 14.0],
			texture_offset: (0, 0),
			hidden,
		},
		Cube {
			from: [lock_x, 7.0, 15.0],
			size: [lock_width, 4.0, 1.0],
			texture_offset: (0, 0),
			hidden,
		},
	]
}

#[cfg(test)]
mod tests {
	use super::*;

	fn chest(name: &str, properties: &[(&str, &str)]) -> BlockState {
		BlockState {
			name: name.into(),
			properties: properties.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
		}
	}

	#[test]
	fn single_and_double_halves_draw_their_boxes_and_leave_out_the_shared_face() {
		assert_eq!(chest_quads(&chest("minecraft:chest", &[("type", "single")])).map(|quads| quads.len()), Some(18));
		assert_eq!(chest_quads(&chest("minecraft:chest", &[("type", "left")])).map(|quads| quads.len()), Some(15));
		assert_eq!(chest_quads(&chest("minecraft:chest", &[("type", "right")])).map(|quads| quads.len()), Some(15));
	}

	#[test]
	fn a_chest_faces_its_facing_and_names_its_texture() {
		let east = chest_quads(&chest("minecraft:trapped_chest", &[("facing", "east"), ("type", "single")])).unwrap();
		assert!(east.iter().all(|quad| quad.texture == "minecraft:entity/chest/trapped"));
		assert!(east.iter().any(|quad| quad.direction == Direction::East), "the lock turns from the south to the east");
		let left = chest_quads(&chest("minecraft:copper_chest", &[("facing", "north"), ("type", "left")])).unwrap();
		assert!(left.iter().all(|quad| quad.texture == "minecraft:entity/chest/copper_left"));
	}

	#[test]
	fn chest_faces_span_the_texture_in_sixteen_units() {
		let quads = chest_quads(&chest("minecraft:chest", &[("type", "single")])).unwrap();
		assert!(quads.iter().flat_map(|quad| quad.uvs).flatten().all(|value| (0.0..=16.0).contains(&value)));
	}

	#[test]
	fn the_lid_top_takes_the_light_square_of_the_texture() {
		let quads = chest_quads(&chest("minecraft:chest", &[("type", "single"), ("facing", "south")])).unwrap();
		let top_of_lid = quads.iter().find(|quad| quad.direction == Direction::Up && quad.corners.iter().all(|corner| corner[1] == 14.0)).unwrap();
		let texels: Vec<[f32; 2]> = top_of_lid.uvs.iter().map(|uv| uv.map(|value| value * 4.0)).collect();
		assert!(texels.iter().all(|uv| (28.0..=42.0).contains(&uv[0]) && (0.0..=14.0).contains(&uv[1])), "{texels:?}");
	}

	#[test]
	fn ender_chests_are_always_single_and_other_blocks_are_not_chests() {
		let ender = chest_quads(&chest("minecraft:ender_chest", &[("facing", "south"), ("type", "left")])).unwrap();
		assert_eq!(ender.len(), 18);
		assert!(chest_quads(&chest("minecraft:stone", &[])).is_none());
	}
}
