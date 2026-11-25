mod defaults;
mod light;
pub use light::Light;
mod palette;

use std::collections::BTreeMap;

pub use defaults::BlockDefaults;

use fastnbt::{ByteArray, LongArray};
use light::SectionLight;
use serde::Deserialize;

use crate::error::{Result, WorldmapError};

const SECTION_SIDE: usize = 16;
const BLOCKS_PER_SECTION: usize = SECTION_SIDE * SECTION_SIDE * SECTION_SIDE;
/// DataVersion of 1.16. Before it, a packed block-state entry may straddle two longs.
const PACKED_WITHOUT_SPANNING: i32 = 2566;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockState {
	pub name: String,
	pub properties: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct Section {
	y: i32,
	palette: Vec<BlockState>,
	indices: Vec<u16>,
	biome_palette: Vec<String>,
	biome_indices: Vec<u16>,
	light: SectionLight,
}

impl Section {
	pub fn y(&self) -> i32 {
		self.y
	}

	/// Sky and block light at section-local `x`, `y` and `z`.
	pub fn light_at(&self, x: usize, y: usize, z: usize) -> (u8, u8) {
		(self.light.sky.at(x, y, z), self.light.block.at(x, y, z))
	}

	pub fn palette(&self) -> &[BlockState] {
		&self.palette
	}

	pub fn indices(&self) -> &[u16] {
		&self.indices
	}

	pub fn biome_indices(&self) -> &[u16] {
		&self.biome_indices
	}

	pub fn sky_light(&self) -> &Light {
		&self.light.sky
	}

	pub fn block_light(&self) -> &Light {
		&self.light.block
	}

	pub fn state_at(&self, x: usize, y: usize, z: usize) -> &BlockState {
		&self.palette[usize::from(self.indices[(y << 8) | (z << 4) | x])]
	}

	/// Biome ids present in this section, indexed by [`Self::biome_index_at`]. Empty before 1.18.
	pub fn biome_palette(&self) -> &[String] {
		&self.biome_palette
	}

	/// Index into [`Self::biome_palette`] of the biome cell holding a block, or `None` before 1.18.
	pub fn biome_index_at(&self, x: usize, y: usize, z: usize) -> Option<usize> {
		let cell = ((y >> 2) << 4) | ((z >> 2) << 2) | (x >> 2);
		self.biome_indices.get(cell).map(|index| usize::from(*index))
	}
}

/// A parsed chunk. Sections are kept highest first, so the top of a column is found without a full scan.
#[derive(Debug)]
pub struct Chunk {
	pub x: i32,
	pub z: i32,
	pub data_version: i32,
	sections: Vec<Section>,
}

impl Chunk {
	pub fn parse(nbt: &[u8]) -> Result<Self> {
		let raw: RawChunk = fastnbt::from_bytes(nbt)?;
		let level = raw.level;
		let x = raw
			.x_pos
			.or(level.as_ref().and_then(|level| level.x_pos))
			.ok_or_else(|| WorldmapError::Chunk("chunk has no xPos".into()))?;
		let z = raw
			.z_pos
			.or(level.as_ref().and_then(|level| level.z_pos))
			.ok_or_else(|| WorldmapError::Chunk("chunk has no zPos".into()))?;
		let raw_sections = raw.sections.or_else(|| level.map(|level| level.sections)).unwrap_or_default();
		let spanning = raw.data_version < PACKED_WITHOUT_SPANNING;
		let mut sections = raw_sections
			.into_iter()
			.filter_map(|section| decode_section(section, spanning).transpose())
			.collect::<Result<Vec<_>>>()?;
		sections.sort_by_key(|section| std::cmp::Reverse(section.y));
		Ok(Self {
			x,
			z,
			data_version: raw.data_version,
			sections,
		})
	}

	pub fn sections(&self) -> &[Section] {
		&self.sections
	}

	pub fn fill_default_properties(&mut self, defaults: &BlockDefaults) {
		for section in &mut self.sections {
			for state in &mut section.palette {
				defaults.fill(state);
			}
		}
	}

	pub fn section(&self, section_y: i32) -> Option<&Section> {
		self.sections.iter().find(|section| section.y == section_y)
	}

	/// The block at chunk-local `x` and `z` (0..16) and absolute height `y`; `None` when no section covers it.
	pub fn state_at(&self, x: usize, y: i32, z: usize) -> Option<&BlockState> {
		let section = self.section(y.div_euclid(SECTION_SIDE as i32))?;
		Some(section.state_at(x, y.rem_euclid(SECTION_SIDE as i32) as usize, z))
	}

	/// Biome id at chunk-local `x` and `z` (0..16) and absolute height `y`. `None` before 1.18 or where no section covers `y`.
	pub fn biome_at(&self, x: usize, y: i32, z: usize) -> Option<&str> {
		let section = self.section(y.div_euclid(SECTION_SIDE as i32))?;
		let index = section.biome_index_at(x, y.rem_euclid(SECTION_SIDE as i32) as usize, z)?;
		section.biome_palette.get(index).map(String::as_str)
	}

	/// Sky and block light at chunk-local `x` and `z` (0..16) and absolute height `y`; `None` when no section covers `y`.
	pub fn light_at(&self, x: usize, y: i32, z: usize) -> Option<(u8, u8)> {
		let section = self.section(y.div_euclid(SECTION_SIDE as i32))?;
		Some(section.light_at(x, y.rem_euclid(SECTION_SIDE as i32) as usize, z))
	}

	/// Highest non-air block in a column, as (world y, block). `x` and `z` are section-local, 0..16.
	pub fn column_top(&self, x: usize, z: usize) -> Option<(i32, &BlockState)> {
		self.sections.iter().find_map(|section| {
			(0..SECTION_SIDE).rev().find_map(|local_y| {
				let state = section.state_at(x, local_y, z);
				(!is_air(&state.name)).then(|| (section.y * SECTION_SIDE as i32 + local_y as i32, state))
			})
		})
	}
}

pub fn is_air(name: &str) -> bool {
	matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

fn decode_section(raw: RawSection, spanning: bool) -> Result<Option<Section>> {
	let (palette, longs) = match (raw.block_states, raw.palette) {
		(Some(container), _) => (container.palette, container.data),
		(None, Some(palette)) => (palette, raw.packed),
		(None, None) => return Ok(None),
	};
	if palette.is_empty() {
		return Ok(None);
	}
	let longs = longs.map(LongArray::into_inner).unwrap_or_default();
	let palette: Vec<BlockState> = palette.into_iter().map(RawBlockState::into_block_state).collect();
	let indices = if palette.len() == 1 {
		vec![0; BLOCKS_PER_SECTION]
	} else {
		palette::unpack_blocks(&longs, palette.len(), spanning)
	};
	if indices.iter().any(|index| usize::from(*index) >= palette.len()) {
		return Err(WorldmapError::Chunk(format!("section {} points past its palette", raw.y)));
	}
	let (biome_palette, biome_indices) = match raw.biomes {
		Some(biomes) => decode_biomes(biomes)?,
		None => (Vec::new(), Vec::new()),
	};
	Ok(Some(Section {
		y: i32::from(raw.y),
		palette,
		indices,
		biome_palette,
		biome_indices,
		light: SectionLight::new(raw.sky_light, raw.block_light),
	}))
}

fn decode_biomes(raw: RawBiomes) -> Result<(Vec<String>, Vec<u16>)> {
	if raw.palette.is_empty() {
		return Ok((Vec::new(), Vec::new()));
	}
	let indices = if raw.palette.len() == 1 {
		vec![0; palette::BIOME_CELLS]
	} else {
		let longs = raw.data.map(LongArray::into_inner).unwrap_or_default();
		palette::unpack_biomes(&longs, raw.palette.len())
	};
	if indices.iter().any(|index| usize::from(*index) >= raw.palette.len()) {
		return Err(WorldmapError::Chunk("biome cells point past their palette".into()));
	}
	Ok((raw.palette, indices))
}

#[derive(Deserialize)]
struct RawChunk {
	#[serde(rename = "DataVersion", default)]
	data_version: i32,
	#[serde(rename = "xPos", default)]
	x_pos: Option<i32>,
	#[serde(rename = "zPos", default)]
	z_pos: Option<i32>,
	#[serde(default)]
	sections: Option<Vec<RawSection>>,
	#[serde(rename = "Level", default)]
	level: Option<RawLevel>,
}

/// Chunks from 1.13 to 1.17 nest everything under `Level`.
#[derive(Deserialize)]
struct RawLevel {
	#[serde(rename = "xPos", default)]
	x_pos: Option<i32>,
	#[serde(rename = "zPos", default)]
	z_pos: Option<i32>,
	#[serde(rename = "Sections", default)]
	sections: Vec<RawSection>,
}

#[derive(Deserialize)]
struct RawSection {
	#[serde(rename = "Y")]
	y: i8,
	/// 1.18 and later.
	#[serde(default)]
	block_states: Option<RawContainer>,
	#[serde(default)]
	biomes: Option<RawBiomes>,
	/// 1.13 to 1.17 keep the palette and packed data beside `Y`.
	#[serde(rename = "Palette", default)]
	palette: Option<Vec<RawBlockState>>,
	#[serde(rename = "BlockStates", default)]
	packed: Option<LongArray>,
	#[serde(rename = "SkyLight", default)]
	sky_light: Option<ByteArray>,
	#[serde(rename = "BlockLight", default)]
	block_light: Option<ByteArray>,
}

#[derive(Deserialize)]
struct RawContainer {
	palette: Vec<RawBlockState>,
	#[serde(default)]
	data: Option<LongArray>,
}

#[derive(Deserialize)]
struct RawBiomes {
	palette: Vec<String>,
	#[serde(default)]
	data: Option<LongArray>,
}

/// Palette entries come in four shapes: a bare name, a `{"": name}` wrapper, 26.x's `{id, properties}`
/// compound, and the classic `{Name, Properties}` compound used from 1.13 to 1.21.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawBlockState {
	Name(String),
	Anonymous {
		#[serde(rename = "")]
		name: String,
	},
	Compact {
		id: String,
		#[serde(default)]
		properties: BTreeMap<String, String>,
	},
	Classic {
		#[serde(rename = "Name")]
		name: String,
		#[serde(rename = "Properties", default)]
		properties: BTreeMap<String, String>,
	},
}

impl RawBlockState {
	fn into_block_state(self) -> BlockState {
		let (name, properties) = match self {
			Self::Name(name) | Self::Anonymous { name } => (name, BTreeMap::new()),
			Self::Compact { id: name, properties } | Self::Classic { name, properties } => (name, properties),
		};
		BlockState { name, properties }
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use fastnbt::Value;

	use super::*;
	use crate::test_support::{chunk_nbt, chunk_nbt_bare, chunk_nbt_with_biomes};

	#[test]
	fn reads_every_palette_entry_shape() {
		let compact = Value::Compound(HashMap::from([
			("id".to_owned(), Value::String("minecraft:oak_leaves".into())),
			("properties".to_owned(), Value::Compound(HashMap::from([("distance".to_owned(), Value::String("4".into()))]))),
		]));
		let entries = [
			Value::String("minecraft:air".into()),
			Value::Compound(HashMap::from([("".to_owned(), Value::String("minecraft:stone".into()))])),
			compact,
			Value::Compound(HashMap::from([("Name".to_owned(), Value::String("minecraft:dirt".into()))])),
		];
		let states: Vec<BlockState> = entries
			.iter()
			.map(|entry| fastnbt::from_value::<RawBlockState>(entry).unwrap().into_block_state())
			.collect();
		let names: Vec<&str> = states.iter().map(|state| state.name.as_str()).collect();
		assert_eq!(names, ["minecraft:air", "minecraft:stone", "minecraft:oak_leaves", "minecraft:dirt"]);
		assert_eq!(states[2].properties.get("distance").map(String::as_str), Some("4"));
	}

	#[test]
	fn accepts_bare_palette_names_written_by_newer_versions() {
		let nbt = chunk_nbt_bare(0, 0, 4000, &[((2, 4, 2), "minecraft:oak_log")]);
		let chunk = Chunk::parse(&nbt).unwrap();
		assert_eq!(chunk.column_top(2, 2).map(|(y, state)| (y, state.name.as_str())), Some((4, "minecraft:oak_log")));
	}

	#[test]
	fn finds_the_highest_solid_block_in_a_column() {
		let nbt = chunk_nbt(3, -2, 3955, &[((0, 0, 0), "minecraft:stone"), ((0, 1, 0), "minecraft:grass_block")]);
		let chunk = Chunk::parse(&nbt).unwrap();
		assert_eq!((chunk.x, chunk.z), (3, -2));
		let (y, state) = chunk.column_top(0, 0).unwrap();
		assert_eq!((y, state.name.as_str()), (1, "minecraft:grass_block"));
		assert!(chunk.column_top(1, 0).is_none());
	}

	#[test]
	fn treats_cave_air_as_empty() {
		let nbt = chunk_nbt(0, 0, 3955, &[((5, 3, 5), "minecraft:cave_air"), ((5, 2, 5), "minecraft:dirt")]);
		let chunk = Chunk::parse(&nbt).unwrap();
		assert_eq!(chunk.column_top(5, 5).map(|(y, state)| (y, state.name.as_str())), Some((2, "minecraft:dirt")));
	}

	#[test]
	fn reads_biomes_per_four_block_cell() {
		let nbt = chunk_nbt_with_biomes(0, 0, 4000, &[], &[((1, 0, 0), "minecraft:desert"), ((0, 1, 1), "minecraft:swamp")]);
		let chunk = Chunk::parse(&nbt).unwrap();
		assert_eq!(chunk.biome_at(4, 0, 0), Some("minecraft:desert"));
		assert_eq!(chunk.biome_at(7, 3, 3), Some("minecraft:desert"));
		assert_eq!(chunk.biome_at(8, 0, 0), Some("minecraft:plains"));
		assert_eq!(chunk.biome_at(0, 4, 4), Some("minecraft:swamp"));
	}

	#[test]
	fn has_no_biomes_without_a_biome_palette() {
		let chunk = Chunk::parse(&chunk_nbt(0, 0, 3955, &[((0, 0, 0), "minecraft:stone")])).unwrap();
		assert_eq!(chunk.biome_at(0, 0, 0), None);
	}
}
