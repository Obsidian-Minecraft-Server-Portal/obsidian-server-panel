//! Palette indices are packed `bits` wide, least significant bits first. Before 1.16 an entry may
//! straddle two longs; from 1.16 entries never cross a long boundary.

const BLOCKS_PER_SECTION: usize = 4096;
/// Biomes are stored per 4x4x4 cell.
pub const BIOME_CELLS: usize = 64;

pub fn unpack_blocks(longs: &[i64], palette_len: usize, spanning: bool) -> Vec<u16> {
	unpack(longs, BLOCKS_PER_SECTION, bits_per_entry(palette_len, 4), spanning)
}

/// Biome entries never straddle longs, in every version that stores biomes per section.
pub fn unpack_biomes(longs: &[i64], palette_len: usize) -> Vec<u16> {
	unpack(longs, BIOME_CELLS, bits_per_entry(palette_len, 1), false)
}

fn unpack(longs: &[i64], entries: usize, bits: usize, spanning: bool) -> Vec<u16> {
	let mask = (1_u64 << bits) - 1;
	let word = |index: usize| longs.get(index).map_or(0, |value| *value as u64);
	(0..entries)
		.map(|entry| {
			let value = if spanning {
				let bit = entry * bits;
				let offset = bit % 64;
				let low = word(bit / 64) >> offset;
				if offset + bits > 64 {
					low | (word(bit / 64 + 1) << (64 - offset))
				} else {
					low
				}
			} else {
				let per_long = 64 / bits;
				word(entry / per_long) >> ((entry % per_long) * bits)
			};
			(value & mask) as u16
		})
		.collect()
}

fn bits_per_entry(palette_len: usize, minimum: usize) -> usize {
	let needed = (usize::BITS - palette_len.saturating_sub(1).leading_zeros()) as usize;
	needed.max(minimum)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn pack(values: &[u16], bits: usize, spanning: bool) -> Vec<i64> {
		let mut longs = vec![0_u64; BLOCKS_PER_SECTION];
		for (index, value) in values.iter().enumerate() {
			let (word, offset) = if spanning {
				let bit = index * bits;
				(bit / 64, bit % 64)
			} else {
				let per_long = 64 / bits;
				(index / per_long, (index % per_long) * bits)
			};
			longs[word] |= u64::from(*value) << offset;
			if spanning && offset + bits > 64 {
				longs[word + 1] |= u64::from(*value) >> (64 - offset);
			}
		}
		longs.into_iter().map(|long| long as i64).collect()
	}

	#[test]
	fn round_trips_both_packing_modes() {
		let values: Vec<u16> = (0..BLOCKS_PER_SECTION).map(|index| (index % 17) as u16).collect();
		for spanning in [false, true] {
			let longs = pack(&values, 5, spanning);
			assert_eq!(unpack_blocks(&longs, 17, spanning), values, "spanning = {spanning}");
		}
	}

	#[test]
	fn uses_at_least_four_bits_per_block_and_one_per_biome() {
		assert_eq!(bits_per_entry(2, 4), 4);
		assert_eq!(bits_per_entry(17, 4), 5);
		assert_eq!(bits_per_entry(2, 1), 1);
		assert_eq!(bits_per_entry(3, 1), 2);
	}

	#[test]
	fn unpacks_biome_cells_without_straddling_longs() {
		let values: Vec<u16> = (0..BIOME_CELLS).map(|index| (index % 3) as u16).collect();
		let longs = pack(&values, 2, false);
		assert_eq!(unpack_biomes(&longs, 3), values);
	}
}
