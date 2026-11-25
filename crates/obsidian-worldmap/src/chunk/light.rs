//! Sky and block light, stored per section as nibble arrays. A section whose cells all hold one level keeps only that level.

use fastnbt::ByteArray;

const PACKED_LEN: usize = 2048;
const OPEN_SKY: u8 = 15;

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
	fn rejects_arrays_of_the_wrong_size() {
		assert!(Light::from_nibbles(ByteArray::new(vec![0; 10])).is_none());
	}
}
