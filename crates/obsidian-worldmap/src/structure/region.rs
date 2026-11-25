//! Region-level structure work: the states and biomes a region uses, and its sections written as volume records. Chunks are
//! parsed one at a time, so a region never holds more than one chunk's sections beyond the file itself.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::{BiomeTable, StateTable, section_volume};
use crate::chunk::{BlockDefaults, BlockState, Chunk};
use crate::error::Result;
use crate::region::{CHUNKS_PER_SIDE, RegionFile};

#[derive(Default)]
pub struct Vocabulary {
    pub states: BTreeSet<BlockState>,
    pub biomes: BTreeSet<String>,
}

pub fn vocabulary(region: &Path, defaults: &BlockDefaults) -> Result<Vocabulary> {
    let mut vocabulary = Vocabulary::default();
    for_each_chunk(region, defaults, |chunk| {
        for section in chunk.sections() {
            vocabulary.states.extend(section.palette().iter().cloned());
            vocabulary.biomes.extend(section.biome_palette().iter().cloned());
        }
        Ok(())
    })?;
    Ok(vocabulary)
}

/// Where a section's volume record is kept, relative to its dimension folder.
pub fn section_path(chunk_x: i32, section_y: i32, chunk_z: i32) -> String {
    let region = format!("r.{}.{}", chunk_x.div_euclid(32), chunk_z.div_euclid(32));
    format!("structure/sections/{region}/section_{chunk_x}_{section_y}_{chunk_z}.obsv")
}

/// Writes one `section_<chunk x>_<y>_<chunk z>.obsv` per section into `dir`, and returns how many were written.
pub fn write_sections(region: &Path, dir: &Path, defaults: &BlockDefaults, states: &StateTable, biomes: &BiomeTable) -> Result<usize> {
    let mut written = 0;
    fs::create_dir_all(dir)?;
    for_each_chunk(region, defaults, |chunk| {
        for section in chunk.sections() {
            let origin = [chunk.x * 16, section.y() * 16, chunk.z * 16];
            let volume = section_volume(section, origin, states, biomes)?;
            let name = format!("section_{}_{}_{}.obsv", chunk.x, section.y(), chunk.z);
            fs::write(dir.join(name), volume.encode())?;
            written += 1;
        }
        Ok(())
    })?;
    Ok(written)
}

fn for_each_chunk(region: &Path, defaults: &BlockDefaults, mut visit: impl FnMut(&Chunk) -> Result<()>) -> Result<()> {
    let region = RegionFile::open(region)?;
    for local_z in 0..CHUNKS_PER_SIDE {
        for local_x in 0..CHUNKS_PER_SIDE {
            let Some(nbt) = region.chunk_nbt(local_x, local_z)? else { continue };
            let mut chunk = Chunk::parse(&nbt)?;
            chunk.fill_default_properties(defaults);
            visit(&chunk)?;
        }
    }
    Ok(())
}
