//! Builds synthetic chunks so tests do not depend on anyone's world files.

use fastnbt::LongArray;
use serde::Serialize;

#[derive(Serialize)]
struct Root {
	#[serde(rename = "DataVersion")]
	data_version: i32,
	#[serde(rename = "xPos")]
	x_pos: i32,
	#[serde(rename = "zPos")]
	z_pos: i32,
	sections: Vec<Section>,
}

#[derive(Serialize)]
struct Section {
	#[serde(rename = "Y")]
	y: i8,
	block_states: Container,
}

#[derive(Serialize)]
struct Container {
	palette: Vec<Entry>,
	data: LongArray,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Entry {
	Bare(String),
	Full {
		#[serde(rename = "Name")]
		name: String,
	},
}

/// One section at Y = 0 holding `blocks` given as section-local (x, y, z) and a block id. Every other block is air.
/// Palette entries are compounds, as 1.13 to 1.21 write them.
pub fn chunk_nbt(x: i32, z: i32, data_version: i32, blocks: &[((usize, usize, usize), &str)]) -> Vec<u8> {
	build(x, z, data_version, blocks, false)
}

/// Same chunk with bare palette names, as 26.x writes blocks that have no properties.
pub fn chunk_nbt_bare(x: i32, z: i32, data_version: i32, blocks: &[((usize, usize, usize), &str)]) -> Vec<u8> {
	build(x, z, data_version, blocks, true)
}

fn build(x: i32, z: i32, data_version: i32, blocks: &[((usize, usize, usize), &str)], bare: bool) -> Vec<u8> {
	let mut palette = vec!["minecraft:air".to_owned()];
	let mut indices = vec![0_u16; 4096];
	for &((block_x, block_y, block_z), name) in blocks {
		let index = match palette.iter().position(|entry| entry.as_str() == name) {
			Some(index) => index,
			None => {
				palette.push(name.to_owned());
				palette.len() - 1
			}
		};
		indices[(block_y << 8) | (block_z << 4) | block_x] = index as u16;
	}
	let root = Root {
		data_version,
		x_pos: x,
		z_pos: z,
		sections: vec![Section {
			y: 0,
			block_states: Container {
				palette: palette
					.into_iter()
					.map(|name| if bare { Entry::Bare(name) } else { Entry::Full { name } })
					.collect(),
				data: LongArray::new(pack(&indices, 4)),
			},
		}],
	};
	fastnbt::to_bytes(&root).expect("synthetic chunk serializes")
}

fn pack(indices: &[u16], bits: usize) -> Vec<i64> {
	let per_long = 64 / bits;
	let mut longs = vec![0_u64; indices.len().div_ceil(per_long)];
	for (position, index) in indices.iter().enumerate() {
		longs[position / per_long] |= u64::from(*index) << ((position % per_long) * bits);
	}
	longs.into_iter().map(|long| long as i64).collect()
}
