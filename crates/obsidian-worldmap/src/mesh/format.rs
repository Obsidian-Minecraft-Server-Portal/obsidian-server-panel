//! Binary section mesh, OBSM version 2, little-endian. The viewer decodes this layout directly.
//!
//! Header (32 bytes): magic `OBSM`, version u16, reserved u16, section chunk x, section y and chunk z as i32,
//! then the quad count of each render layer (opaque, cutout, translucent) as u32.
//! Quads follow in layer order, 48 bytes each: four corners of (x i16, y i16, z i16, u u16, v u16), then the
//! face direction (u8: down, up, north, south, west, east), the tint colour (r, g, b) and the brightness of each
//! corner (u8 x 4). Positions are in 1/256 of a block from the section origin. UVs are atlas coordinates in image
//! space, scaled to 0..=65535.

use crate::error::{Result, WorldmapError};
use crate::mesh::{MeshQuad, SectionMesh};
use crate::models::geometry::Direction;

pub const MAGIC: [u8; 4] = *b"OBSM";
pub const VERSION: u16 = 2;
pub const HEADER_LEN: usize = 32;
pub const QUAD_LEN: usize = 48;

pub fn encode(mesh: &SectionMesh) -> Vec<u8> {
	let quads: usize = mesh.layers.iter().map(Vec::len).sum();
	let mut out = Vec::with_capacity(HEADER_LEN + quads * QUAD_LEN);
	out.extend_from_slice(&MAGIC);
	out.extend_from_slice(&VERSION.to_le_bytes());
	out.extend_from_slice(&0_u16.to_le_bytes());
	for coordinate in mesh.section {
		out.extend_from_slice(&coordinate.to_le_bytes());
	}
	for layer in &mesh.layers {
		out.extend_from_slice(&(layer.len() as u32).to_le_bytes());
	}
	for quad in mesh.layers.iter().flatten() {
		for (corner, uv) in quad.corners.iter().zip(&quad.uvs) {
			for value in corner {
				out.extend_from_slice(&value.to_le_bytes());
			}
			for value in uv {
				out.extend_from_slice(&value.to_le_bytes());
			}
		}
		out.push(quad.direction.index());
		out.extend_from_slice(&quad.tint);
		out.extend_from_slice(&quad.light);
	}
	out
}

pub fn decode(bytes: &[u8]) -> Result<SectionMesh> {
	let header = bytes.get(..HEADER_LEN).ok_or_else(|| mesh_error("shorter than its header"))?;
	if header[..4] != MAGIC {
		return Err(mesh_error("bad magic"));
	}
	let version = u16::from_le_bytes([header[4], header[5]]);
	if version != VERSION {
		return Err(mesh_error(&format!("unsupported version {version}")));
	}
	let section = [0, 1, 2].map(|index| read_i32(header, 8 + index * 4));
	let counts = [0, 1, 2].map(|index| read_u32(header, 20 + index * 4) as usize);
	let mut cursor = HEADER_LEN;
	let mut layers: [Vec<MeshQuad>; 3] = Default::default();
	for (layer, count) in layers.iter_mut().zip(counts) {
		for _ in 0..count {
			let record = bytes
				.get(cursor..cursor + QUAD_LEN)
				.ok_or_else(|| mesh_error("truncated quad list"))?;
			layer.push(read_quad(record)?);
			cursor += QUAD_LEN;
		}
	}
	Ok(SectionMesh { section, layers })
}

fn read_quad(record: &[u8]) -> Result<MeshQuad> {
	let mut corners = [[0_i16; 3]; 4];
	let mut uvs = [[0_u16; 2]; 4];
	for corner in 0..4 {
		let base = corner * 10;
		corners[corner] = [0, 1, 2].map(|axis| read_i16(record, base + axis * 2));
		uvs[corner] = [0, 1].map(|axis| read_u16(record, base + 6 + axis * 2));
	}
	let direction = Direction::from_index(record[40]).ok_or_else(|| mesh_error("unknown face direction"))?;
	Ok(MeshQuad {
		corners,
		uvs,
		direction,
		tint: [record[41], record[42], record[43]],
		light: [record[44], record[45], record[46], record[47]],
	})
}

fn read_i32(bytes: &[u8], at: usize) -> i32 {
	i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
	u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn read_i16(bytes: &[u8], at: usize) -> i16 {
	i16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn read_u16(bytes: &[u8], at: usize) -> u16 {
	u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn mesh_error(reason: &str) -> WorldmapError {
	WorldmapError::Mesh(reason.to_owned())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn sample() -> SectionMesh {
		let quad = MeshQuad {
			corners: [[0, 0, 0], [256, 0, 0], [256, 256, 0], [0, 256, 0]],
			uvs: [[0, 0], [65535, 0], [65535, 65535], [0, 65535]],
			direction: Direction::North,
			tint: [121, 176, 66],
			light: [255, 204, 170, 96],
		};
		SectionMesh {
			section: [-3, 4, 9],
			layers: [vec![quad.clone(), quad.clone()], vec![], vec![quad]],
		}
	}

	#[test]
	fn round_trips_every_field() {
		let mesh = sample();
		let bytes = encode(&mesh);
		assert_eq!(bytes.len(), HEADER_LEN + 3 * QUAD_LEN);
		assert_eq!(decode(&bytes).unwrap(), mesh);
	}

	#[test]
	fn rejects_bad_magic_and_truncated_data() {
		let mut bytes = encode(&sample());
		bytes[0] = b'X';
		assert!(decode(&bytes).is_err());
		let bytes = encode(&sample());
		assert!(decode(&bytes[..bytes.len() - 1]).is_err());
	}
}
