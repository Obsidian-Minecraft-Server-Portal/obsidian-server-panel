mod defaults;
mod light;
pub use light::Light;
mod palette;

use std::collections::BTreeMap;

pub use defaults::BlockDefaults;

use fastnbt::{ByteArray, LongArray};
use light::{OPEN_SKY, SectionLight};
use serde::Deserialize;

use crate::error::{Result, WorldmapError};
use crate::region::{CHUNKS_PER_SIDE, RegionFile};

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
		raise_sky_to_open_columns(&mut sections);
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

	/// Highest visible block in a column, as (world y, block). `x` and `z` are section-local, 0..16.
	pub fn column_top(&self, x: usize, z: usize) -> Option<(i32, &BlockState)> {
		self.column_top_where(x, z, |_| true)
	}

	/// Highest block in a column that is visible and accepted by `keep`, as (world y, block).
	pub fn column_top_where(&self, x: usize, z: usize, mut keep: impl FnMut(&BlockState) -> bool) -> Option<(i32, &BlockState)> {
		self.sections.iter().find_map(|section| {
			(0..SECTION_SIDE).rev().find_map(|local_y| {
				let state = section.state_at(x, local_y, z);
				(!is_invisible(&state.name) && keep(state)).then(|| (section.y * SECTION_SIDE as i32 + local_y as i32, state))
			})
		})
	}
}

/// Raises each cell's sky light to what open sky reaches straight down its column. A chunk lit before its neighbours existed can
/// store too little light beside its edges, which the map would draw as a dark band.
fn raise_sky_to_open_columns(sections: &mut [Section]) {
	let mut columns = [OPEN_SKY; SECTION_SIDE * SECTION_SIDE];
	let floors: Vec<Vec<u8>> = sections
		.iter()
		.map(|section| {
			let reductions: Vec<Option<u8>> = section.palette.iter().map(|state| light_reduction(&state.name)).collect();
			let mut floor = vec![0_u8; BLOCKS_PER_SECTION];
			for y in (0..SECTION_SIDE).rev() {
				for z in 0..SECTION_SIDE {
					for x in 0..SECTION_SIDE {
						let index = (y << 8) | (z << 4) | x;
						let level = &mut columns[(z << 4) | x];
						match reductions[usize::from(section.indices[index])] {
							Some(reduction) => {
								*level = level.saturating_sub(reduction);
								floor[index] = *level;
							}
							None => *level = 0,
						}
					}
				}
			}
			floor
		})
		.collect();
	for (section, floor) in sections.iter_mut().zip(floors) {
		let stored: Vec<u8> = (0..BLOCKS_PER_SECTION)
			.map(|index| section.light.sky.at(index & 15, index >> 8, (index >> 4) & 15))
			.collect();
		let raised: Vec<u8> = stored.iter().zip(&floor).map(|(&sky, &open)| sky.max(open)).collect();
		if raised != stored {
			section.light.sky = Light::from_levels(&raised);
		}
	}
}

/// How much a block dims the sky light that passes through it. `None` blocks it, which keeps the floor low wherever unsure.
fn light_reduction(name: &str) -> Option<u8> {
	if is_invisible(name) || name.ends_with("glass") {
		Some(0)
	} else if name == "minecraft:water" || name.ends_with("_leaves") {
		Some(1)
	} else {
		None
	}
}

/// Blocks that draw nothing in game, so the map treats them as empty.
pub fn is_invisible(name: &str) -> bool {
	matches!(
		name,
		"minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:barrier" | "minecraft:light" | "minecraft:structure_void"
	)
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

/// The chunk at `local_x`, `local_z` of `region`, or `None` when it was never generated or cannot be read. A chunk that cannot
/// be read is logged and treated as missing, so one damaged chunk never costs the rest of its region.
pub fn read_chunk(region: &RegionFile, local_x: usize, local_z: usize) -> Option<Chunk> {
	let (region_x, region_z) = region.region_coords();
	match region.chunk_nbt(local_x, local_z).and_then(|nbt| nbt.map(|nbt| Chunk::parse(&nbt)).transpose()) {
		Ok(chunk) => chunk,
		Err(error) => {
			tracing::warn!(region_x, region_z, local_x, local_z, %error, "skipping an unreadable chunk");
			None
		}
	}
}

pub fn load_chunks(region: &RegionFile) -> Vec<Chunk> {
	let mut chunks = Vec::new();
	for local_z in 0..CHUNKS_PER_SIDE {
		for local_x in 0..CHUNKS_PER_SIDE {
			chunks.extend(read_chunk(region, local_x, local_z));
		}
	}
	chunks
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use fastnbt::Value;

	use super::*;
	use crate::test_support::{chunk_nbt, chunk_nbt_bare, chunk_nbt_with_biomes, region_records, zlib};

	#[test]
	fn a_damaged_chunk_is_skipped_and_its_neighbours_still_load() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("r.0.0.mca");
		let good = chunk_nbt(0, 0, 3955, &[((0, 0, 0), "minecraft:stone")]);
		std::fs::write(&path, region_records(&[(0, 0, zlib(&good)), (1, 0, vec![0xde, 0xad, 0xbe, 0xef])])).unwrap();
		let chunks = load_chunks(&RegionFile::open(&path).unwrap());
		assert_eq!(chunks.iter().map(|chunk| (chunk.x, chunk.z)).collect::<Vec<_>>(), [(0, 0)]);
	}

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
	fn treats_invisible_blocks_as_empty() {
		let nbt = chunk_nbt(0, 0, 3955, &[((5, 4, 5), "minecraft:barrier"), ((5, 3, 5), "minecraft:structure_void"), ((5, 2, 5), "minecraft:light"), ((5, 1, 5), "minecraft:dirt")]);
		let chunk = Chunk::parse(&nbt).unwrap();
		assert_eq!(chunk.column_top(5, 5).map(|(y, state)| (y, state.name.as_str())), Some((1, "minecraft:dirt")));
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

	fn air() -> BlockState {
		BlockState {
			name: "minecraft:air".into(),
			properties: BTreeMap::new(),
		}
	}

	fn section(y: i32, palette: Vec<BlockState>, indices: Vec<u16>) -> Section {
		Section {
			y,
			palette,
			indices,
			biome_palette: Vec::new(),
			biome_indices: Vec::new(),
			light: SectionLight {
				sky: Light::Uniform(0),
				block: Light::Uniform(0),
			},
		}
	}

	#[test]
	fn sky_light_is_raised_to_what_open_sky_reaches_down_each_column() {
		let stone = BlockState {
			name: "minecraft:stone".into(),
			properties: BTreeMap::new(),
		};
		let mut lower = vec![0_u16; BLOCKS_PER_SECTION];
		lower[15 << 8] = 1;
		let mut sections = vec![section(1, vec![air()], vec![0; BLOCKS_PER_SECTION]), section(0, vec![air(), stone], lower)];
		raise_sky_to_open_columns(&mut sections);
		assert_eq!(sections[0].light_at(3, 3, 3).0, OPEN_SKY);
		assert_eq!(sections[1].light_at(1, 15, 0).0, OPEN_SKY);
		assert_eq!(sections[1].light_at(0, 15, 0).0, 0);
		assert_eq!(sections[1].light_at(0, 14, 0).0, 0);
	}
}
