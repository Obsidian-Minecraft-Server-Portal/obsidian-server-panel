//! Sky and block light, stored per section as nibble arrays. A section whose cells all hold one level keeps only that level.

use fastnbt::ByteArray;

const PACKED_LEN: usize = 2048;
/// The level open sky gives every cell that nothing covers.
pub const OPEN_SKY: u8 = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Light {
	Uniform(u8),
	Packed(Box<[u8; PACKED_LEN]>),
}

impl Light {
	/// Nibbles in index order `y << 8 | z << 4 | x`, low nibble first. `None` when the array is not a section's size.
	fn from_nibbles(bytes: ByteArray) -> Option<Self> {
		let raw: Vec<u8> = bytes.into_inner().into_iter().map(|byte| byte as u8).collect();
		let packed: Box<[u8; PACKED_LEN]> = raw.into_boxed_slice().try_into().ok()?;
		let first = packed[0];
		if packed.iter().all(|&byte| byte == first) && first >> 4 == first & 0x0F {
			return Some(Self::Uniform(first & 0x0F));
		}
		Some(Self::Packed(packed))
	}

	/// The levels of every cell in the index order `at` reads, which collapse to one level when they all share it.
	pub fn from_levels(levels: &[u8]) -> Self {
		let first = levels[0];
		if levels.iter().all(|&level| level == first) {
			return Self::Uniform(first);
		}
		let mut packed = Box::new([0_u8; PACKED_LEN]);
		for (byte, pair) in packed.iter_mut().zip(levels.as_chunks::<2>().0) {
			*byte = pair[0] | (pair[1] << 4);
		}
		Self::Packed(packed)
	}

	pub fn at(&self, x: usize, y: usize, z: usize) -> u8 {
		match self {
			Self::Uniform(level) => *level,
			Self::Packed(bytes) => {
				let index = (y << 8) | (z << 4) | x;
				let byte = bytes[index / 2];
				if index.is_multiple_of(2) { byte & 0x0F } else { byte >> 4 }
			}
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionLight {
	pub sky: Light,
	pub block: Light,
}

impl SectionLight {
	/// Missing arrays mean open sky and no block light, which is what chunks without light data are drawn with.
	pub fn new(sky: Option<ByteArray>, block: Option<ByteArray>) -> Self {
		Self {
			sky: sky.and_then(Light::from_nibbles).unwrap_or(Light::Uniform(OPEN_SKY)),
			block: block.and_then(Light::from_nibbles).unwrap_or(Light::Uniform(0)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_single_level_collapses_to_one_value() {
		let light = Light::from_nibbles(ByteArray::new(vec![-1; PACKED_LEN])).unwrap();
		assert_eq!(light, Light::Uniform(15));
		assert_eq!(light.at(3, 7, 2), 15);
	}

	#[test]
	fn packed_cells_read_the_low_nibble_first() {
		let mut bytes = vec![-1_i8; PACKED_LEN];
		bytes[0] = 0x3A_u8 as i8;
		let light = Light::from_nibbles(ByteArray::new(bytes)).unwrap();
		assert_eq!((light.at(0, 0, 0), light.at(1, 0, 0)), (10, 3));
	}

	#[test]
	fn levels_round_trip_through_the_packed_form() {
		let levels: Vec<u8> = (0..4096).map(|index| (index % 16) as u8).collect();
		let light = Light::from_levels(&levels);
		assert_eq!(light.at(5, 3, 9), levels[(3 << 8) | (9 << 4) | 5]);
		assert_eq!(Light::from_levels(&[7; 4096]), Light::Uniform(7));
	}

	#[test]
	fn rejects_arrays_of_the_wrong_size() {
		assert!(Light::from_nibbles(ByteArray::new(vec![0; 10])).is_none());
	}
}
