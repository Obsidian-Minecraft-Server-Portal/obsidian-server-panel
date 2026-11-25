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

/// A region file holding `chunks`, each given by its local (x, z) and NBT bytes. Chunks are zlib-compressed.
pub fn region_bytes(chunks: &[(usize, usize, Vec<u8>)]) -> Vec<u8> {
	let mut header = vec![0_u8; 8192];
	let mut body = Vec::new();
	let mut sector = 2_u32;
	for (local_x, local_z, nbt) in chunks {
		let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
		encoder.write_all(nbt).unwrap();
		let compressed = encoder.finish().unwrap();
		let mut record = ((compressed.len() + 1) as u32).to_be_bytes().to_vec();
		record.push(2);
		record.extend(compressed);
		let sectors = record.len().div_ceil(4096);
		record.resize(sectors * 4096, 0);
		let entry = local_z * 32 + local_x;
		header[entry * 4..entry * 4 + 4].copy_from_slice(&((sector << 8) | sectors as u32).to_be_bytes());
		body.extend(record);
		sector += sectors as u32;
	}
	header.extend(body);
	header
}

/// Assets for one textured cube, so block meshes and atlases have something to pack.
pub fn stone_assets() -> crate::assets::AssetStore {
	let faces = r##"{"down": {"texture": "#all", "cullface": "down"}, "up": {"texture": "#all", "cullface": "up"}, "north": {"texture": "#all", "cullface": "north"}, "south": {"texture": "#all", "cullface": "south"}, "west": {"texture": "#all", "cullface": "west"}, "east": {"texture": "#all", "cullface": "east"}}"##;
	let model = format!(r##"{{"textures": {{"all": "minecraft:block/stone"}}, "elements": [{{"from": [0,0,0], "to": [16,16,16], "faces": {faces}}}]}}"##);
	let mut assets = crate::assets::AssetStore::default();
	assets.insert("minecraft".into(), "blockstates/stone.json".into(), br#"{"variants": {"": {"model": "minecraft:block/cube_all"}}}"#.to_vec());
	assets.insert("minecraft".into(), "models/block/cube_all.json".into(), model.into_bytes());
	let mut png = Vec::new();
	image::RgbaImage::from_pixel(16, 16, image::Rgba([120, 120, 120, 255]))
		.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
		.unwrap();
	assets.insert("minecraft".into(), "textures/block/stone.png".into(), png);
	assets
}
