//! Builds synthetic chunks and jars so tests do not depend on anyone's world files.

use std::io::{Cursor, Write};
use std::path::Path;

use fastnbt::LongArray;
use serde::Serialize;

type Cells<'a> = &'a [((usize, usize, usize), &'a str)];

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
	#[serde(skip_serializing_if = "Option::is_none")]
	biomes: Option<Biomes>,
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

#[derive(Serialize)]
struct Biomes {
	palette: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	data: Option<LongArray>,
}

/// One section at Y = 0 holding `blocks` given as section-local (x, y, z) and a block id. Every other block is air.
/// Palette entries are compounds, as 1.13 to 1.21 write them.
pub fn chunk_nbt(x: i32, z: i32, data_version: i32, blocks: &[((usize, usize, usize), &str)]) -> Vec<u8> {
	build(x, z, data_version, blocks, false, None)
}

/// Same chunk with bare palette names, as 26.x writes blocks that have no properties.
pub fn chunk_nbt_bare(x: i32, z: i32, data_version: i32, blocks: &[((usize, usize, usize), &str)]) -> Vec<u8> {
	build(x, z, data_version, blocks, true, None)
}

/// Same chunk with biomes given per 4x4x4 cell. Cells not listed are `minecraft:plains`.
pub fn chunk_nbt_with_biomes(x: i32, z: i32, data_version: i32, blocks: Cells<'_>, biomes: Cells<'_>) -> Vec<u8> {
	build(x, z, data_version, blocks, false, Some(biomes))
}

fn build(x: i32, z: i32, data_version: i32, blocks: Cells<'_>, bare: bool, biomes: Option<Cells<'_>>) -> Vec<u8> {
	let mut palette = vec!["minecraft:air".to_owned()];
	let mut indices = vec![0_u16; 4096];
	for &((block_x, block_y, block_z), name) in blocks {
		indices[(block_y << 8) | (block_z << 4) | block_x] = intern(&mut palette, name) as u16;
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
			biomes: biomes.map(biome_container),
		}],
	};
	fastnbt::to_bytes(&root).expect("synthetic chunk serializes")
}

fn biome_container(cells: &[((usize, usize, usize), &str)]) -> Biomes {
	let mut palette = vec!["minecraft:plains".to_owned()];
	let mut indices = vec![0_u16; 64];
	for &((cell_x, cell_y, cell_z), id) in cells {
		indices[(cell_y << 4) | (cell_z << 2) | cell_x] = intern(&mut palette, id) as u16;
	}
	let bits = (usize::BITS - (palette.len() - 1).leading_zeros()).max(1) as usize;
	let data = (palette.len() > 1).then(|| LongArray::new(pack(&indices, bits)));
	Biomes { palette, data }
}

fn intern(palette: &mut Vec<String>, name: &str) -> usize {
	match palette.iter().position(|entry| entry.as_str() == name) {
		Some(index) => index,
		None => {
			palette.push(name.to_owned());
			palette.len() - 1
		}
	}
}

fn pack(indices: &[u16], bits: usize) -> Vec<i64> {
	let per_long = 64 / bits;
	let mut longs = vec![0_u64; indices.len().div_ceil(per_long)];
	for (position, index) in indices.iter().enumerate() {
		longs[position / per_long] |= u64::from(*index) << ((position % per_long) * bits);
	}
	longs.into_iter().map(|long| long as i64).collect()
}

pub fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
	let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
	for (name, bytes) in entries {
		writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
		writer.write_all(bytes).unwrap();
	}
	writer.finish().unwrap().into_inner()
}

pub fn write_jar(path: &Path, entries: &[(&str, &[u8])]) {
	std::fs::write(path, zip_bytes(entries)).unwrap();
}
