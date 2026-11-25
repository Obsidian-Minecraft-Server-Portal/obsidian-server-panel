//! Every section of one cell, packed into a single response, so a viewer fetches a cell with one request. A cell is four
//! chunks by four chunks by four sections: 64 blocks a side, which divides a region evenly.

use std::path::Path;

use obsidian_common::Result;

const CELL_SIDE: i32 = 4;
const REGION_CHUNKS: i32 = 32;

/// `count` as u32, then per section its chunk x, section y and chunk z as i32, its byte length as u32 and its OBSM bytes.
/// Sections the render never wrote are left out.
pub async fn cell(folder_dir: &Path, cell: (i32, i32, i32)) -> Result<Vec<u8>> {
	let mut sections = Vec::new();
	for x in cell.0 * CELL_SIDE..(cell.0 + 1) * CELL_SIDE {
		for z in cell.2 * CELL_SIDE..(cell.2 + 1) * CELL_SIDE {
			for y in cell.1 * CELL_SIDE..(cell.1 + 1) * CELL_SIDE {
				let region = format!("r.{}.{}", x.div_euclid(REGION_CHUNKS), z.div_euclid(REGION_CHUNKS));
				let path = folder_dir.join("sections").join(region).join(format!("section_{x}_{y}_{z}.obsm"));
				if let Ok(bytes) = tokio::fs::read(&path).await {
					sections.push(([x, y, z], bytes));
				}
			}
		}
	}
	Ok(encode(&sections))
}

fn encode(sections: &[([i32; 3], Vec<u8>)]) -> Vec<u8> {
	let payload: usize = sections.iter().map(|(_, bytes)| 16 + bytes.len()).sum();
	let mut out = Vec::with_capacity(4 + payload);
	out.extend_from_slice(&(sections.len() as u32).to_le_bytes());
	for ([x, y, z], bytes) in sections {
		for coordinate in [x, y, z] {
			out.extend_from_slice(&coordinate.to_le_bytes());
		}
		out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
		out.extend_from_slice(bytes);
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn packs_each_section_behind_its_coordinates_and_length() {
		let bytes = encode(&[([-3, 4, 9], vec![1, 2, 3]), ([0, 0, 0], vec![])]);
		assert_eq!(u32::from_le_bytes(bytes[0..4].try_into().unwrap()), 2);
		assert_eq!(i32::from_le_bytes(bytes[4..8].try_into().unwrap()), -3);
		assert_eq!(i32::from_le_bytes(bytes[8..12].try_into().unwrap()), 4);
		assert_eq!(i32::from_le_bytes(bytes[12..16].try_into().unwrap()), 9);
		assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 3);
		assert_eq!(&bytes[20..23], &[1, 2, 3]);
		assert_eq!(bytes.len(), 4 + 16 + 3 + 16);
	}

	#[test]
	fn an_empty_cell_is_a_zero_count() {
		assert_eq!(encode(&[]), 0_u32.to_le_bytes());
	}
}
