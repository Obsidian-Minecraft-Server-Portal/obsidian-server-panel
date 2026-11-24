//! Block-state indices are packed `bits` wide, least significant bits first. Before 1.16 an entry may
//! straddle two longs; from 1.16 entries never cross a long boundary.

const BLOCKS_PER_SECTION: usize = 4096;

pub fn unpack(longs: &[i64], palette_len: usize, spanning: bool) -> Vec<u16> {
	let bits = bits_per_entry(palette_len);
	let mask = (1_u64 << bits) - 1;
	let word = |index: usize| longs.get(index).map_or(0, |value| *value as u64);
	(0..BLOCKS_PER_SECTION)
		.map(|block| {
			let value = if spanning {
				let bit = block * bits;
				let offset = bit % 64;
				let low = word(bit / 64) >> offset;
				if offset + bits > 64 {
					low | (word(bit / 64 + 1) << (64 - offset))
				} else {
					low
				}
			} else {
				let per_long = 64 / bits;
				word(block / per_long) >> ((block % per_long) * bits)
			};
			(value & mask) as u16
		})
		.collect()
}

fn bits_per_entry(palette_len: usize) -> usize {
	let needed = usize::BITS - palette_len.saturating_sub(1).leading_zeros();
	needed.max(4) as usize
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
			assert_eq!(unpack(&longs, 17, spanning), values, "spanning = {spanning}");
		}
	}

	#[test]
	fn uses_at_least_four_bits_per_entry() {
		assert_eq!(bits_per_entry(2), 4);
		assert_eq!(bits_per_entry(16), 4);
		assert_eq!(bits_per_entry(17), 5);
	}
}
