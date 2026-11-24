//! Block lookups across chunk boundaries, so a face on a boundary is culled by the block next door.

use std::collections::HashMap;

use crate::chunk::{BlockDefaults, BlockState, Chunk};

pub struct BlockWorld {
	chunks: HashMap<(i32, i32), Chunk>,
}

impl BlockWorld {
	/// Fills in default properties so bare palette names bake to their default state.
	pub fn new(chunks: impl IntoIterator<Item = Chunk>, defaults: &BlockDefaults) -> Self {
		Self {
			chunks: chunks
				.into_iter()
				.map(|mut chunk| {
					chunk.fill_default_properties(defaults);
					((chunk.x, chunk.z), chunk)
				})
				.collect(),
		}
	}

	pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
		self.chunks.values()
	}

	pub fn chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
		self.chunks.get(&(chunk_x, chunk_z))
	}

	/// The block at absolute block coordinates; `None` when its chunk is not loaded.
	pub fn state(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
		let chunk = self.chunk(x.div_euclid(16), z.div_euclid(16))?;
		chunk.state_at(x.rem_euclid(16) as usize, y, z.rem_euclid(16) as usize)
	}
}
