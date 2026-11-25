//! Block structure for the browser to mesh. The server stores each section as a volume record and sends a cell as a bundle of
//! records with its border, along with tables that give each state's faces and each biome's tints.

mod bits;
mod cell;
mod export;
mod region;
mod section;
mod tables;
mod volume;

pub use cell::{CELL_BLOCKS, SectionStore, assemble, read_bundle};
pub use export::{BiomeEntry, BlockEntry, BlockTable, QuadEntry, biome_table, block_table};
pub use region::{Vocabulary, section_path, vocabulary, write_sections};
pub use section::section_volume;
pub use tables::{AIR_ID, BiomeTable, PLAINS_ID, StateTable};
pub use volume::{BIOME_CELLS, Biomes, Light, Volume};
