//! Brightness of each corner of a face: ambient occlusion from the solid cubes beside the corner, and the light that reaches it.

use std::array;
use std::collections::HashMap;

use super::{Baked, BlockWorld, block_key};
use crate::chunk::is_air;
use crate::models::geometry::{Direction, Quad};

/// Brightness for zero to three occluded sides at a corner, as vanilla shades them.
const OCCLUSION: [f32; 4] = [0.4, 0.6, 0.8, 1.0];
/// The dimmest a cell can get, so unlit caves read as dark rather than black.
const AMBIENT: f32 = 0.05;
const FULL_LIGHT: f32 = 15.0;
const OPEN_SKY: (u8, u8) = (15, 0);

/// Brightness of each corner, 0 to 255, for a face of the cube at `block`.
pub fn corner_brightness(world: &BlockWorld, blocks: &HashMap<String, Baked>, quad: &Quad, block: [i32; 3]) -> [u8; 4] {
	let front = step(block, quad.direction.offset());
	let (first, second) = tangent_axes(quad.direction);
	array::from_fn(|corner| {
		let position = quad.corners[corner];
		let first_side = if position[first] < 8.0 { -1 } else { 1 };
		let second_side = if position[second] < 8.0 { -1 } else { 1 };
		let beside_first = along(front, first, first_side);
		let beside_second = along(front, second, second_side);
		let diagonal = along(beside_first, second, second_side);
		let occluded = [beside_first, beside_second].map(|cell| is_occluder(world, blocks, cell));
		let occlusion = if occluded[0] && occluded[1] {
			0
		} else {
			3 - (occluded[0] as usize + occluded[1] as usize + is_occluder(world, blocks, diagonal) as usize)
		};
		let level = [front, beside_first, beside_second, diagonal].map(|cell| light_level(world, cell)).iter().map(|&level| f32::from(level)).sum::<f32>() / 4.0;
		(OCCLUSION[occlusion] * light_curve(level) * 255.0).round() as u8
	})
}

fn tangent_axes(direction: Direction) -> (usize, usize) {
	match direction {
		Direction::Down | Direction::Up => (0, 2),
		Direction::North | Direction::South => (0, 1),
		Direction::West | Direction::East => (1, 2),
	}
}

fn step(cell: [i32; 3], offset: [i32; 3]) -> [i32; 3] {
	array::from_fn(|axis| cell[axis] + offset[axis])
}

fn along(cell: [i32; 3], axis: usize, delta: i32) -> [i32; 3] {
	let mut moved = cell;
	moved[axis] += delta;
	moved
}

fn is_occluder(world: &BlockWorld, blocks: &HashMap<String, Baked>, cell: [i32; 3]) -> bool {
	world.state(cell[0], cell[1], cell[2]).is_some_and(|state| {
		!is_air(&state.name) && blocks.get(&block_key(state)).is_some_and(|baked| baked.full.iter().all(|&face| face))
	})
}

/// The brighter of sky and block light, since either one lights a cell.
fn light_level(world: &BlockWorld, cell: [i32; 3]) -> u8 {
	let (sky, block) = world.light(cell[0], cell[1], cell[2]).unwrap_or(OPEN_SKY);
	sky.max(block)
}

fn light_curve(level: f32) -> f32 {
	let t = (level / FULL_LIGHT).clamp(0.0, 1.0);
	AMBIENT + (1.0 - AMBIENT) * t / (4.0 - 3.0 * t)
}
