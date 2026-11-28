//! Palette indices packed at 0, 1, 2, 4, 8 or 16 bits per entry. No entry straddles a byte, so a reader needs only shifts.

use crate::error::{Result, WorldmapError};

pub fn width_for(palette_len: usize) -> u8 {
    match palette_len {
        0 | 1 => 0,
        2 => 1,
        3..=4 => 2,
        5..=16 => 4,
        17..=256 => 8,
        _ => 16,
    }
}

pub fn byte_len(count: usize, width: u8) -> usize {
    (count * usize::from(width)).div_ceil(8)
}

pub fn pack(indices: &[u16], width: u8) -> Vec<u8> {
    match width {
        0 => Vec::new(),
        16 => indices.iter().flat_map(|index| index.to_le_bytes()).collect(),
        _ => {
            let bits = usize::from(width);
            let per_byte = 8 / bits;
            let mut bytes = vec![0_u8; byte_len(indices.len(), width)];
            for (position, &index) in indices.iter().enumerate() {
                bytes[position / per_byte] |= (index as u8) << ((position % per_byte) * bits);
            }
            bytes
        }
    }
}

pub fn unpack(bytes: &[u8], width: u8, count: usize) -> Result<Vec<u16>> {
    if bytes.len() < byte_len(count, width) {
        return Err(WorldmapError::Structure("packed indices are truncated".into()));
    }
    Ok(match width {
        0 => vec![0; count],
        16 => bytes
            .as_chunks::<2>().0.iter()
            .take(count)
            .map(|pair| u16::from_le_bytes(*pair))
            .collect(),
        _ => {
            let bits = usize::from(width);
            let per_byte = 8 / bits;
            let mask = (1_u16 << bits) - 1;
            (0..count)
                .map(|position| (u16::from(bytes[position / per_byte] >> ((position % per_byte) * bits))) & mask)
                .collect()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_narrowest_width_for_each_palette_size() {
        assert_eq!([width_for(1), width_for(2), width_for(3), width_for(16), width_for(17), width_for(300)], [0, 1, 2, 4, 8, 16]);
    }

    #[test]
    fn round_trips_every_width() {
        for width in [0_u8, 1, 2, 4, 8, 16] {
            let max = if width == 0 { 1 } else { (1_u32 << width.min(15)) as u16 };
            let indices: Vec<u16> = (0..101).map(|index| (index as u16 * 7) % max).collect();
            let packed = pack(&indices, width);
            assert_eq!(packed.len(), byte_len(indices.len(), width));
            let expected = if width == 0 { vec![0; indices.len()] } else { indices };
            assert_eq!(unpack(&packed, width, expected.len()).unwrap(), expected, "width {width}");
        }
    }

    #[test]
    fn rejects_truncated_data() {
        assert!(unpack(&[0_u8; 3], 8, 4).is_err());
    }
}
