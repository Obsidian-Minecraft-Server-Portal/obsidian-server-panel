//! Block structure for the browser to mesh. The server stores each section as a volume record in its region's pack, and sends a
//! cell as a bundle of records with its border, along with tables that give each state's faces and each biome's tints.

mod bits;
mod cell;
mod export;
mod folder;
mod overlay;
mod pack;
mod region;
mod section;
mod tables;
mod volume;

pub use cell::{CELL_BLOCKS, SectionStore, assemble, read_bundle};
pub use export::{BiomeEntry, BlockEntry, BlockTable, Lighting, QuadEntry, biome_table, block_table, lighting};
pub use folder::{cell_bundle, region_pack_path};
pub use region::{Vocabulary, vocabulary, write_region_pack};
pub use section::section_volume;
pub use tables::{AIR_ID, BiomeTable, PLAINS_ID, StateTable};
pub use volume::{BIOME_CELLS, Biomes, Light, Volume};
