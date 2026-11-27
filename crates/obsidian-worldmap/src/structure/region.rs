//! Region-level structure work: the states and biomes a region uses, and its sections written as one pack. Chunks are parsed
//! one at a time, so a region never holds more than one chunk's sections beyond the file itself.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::cell::OPEN_SKY;
use super::folder::{LEGACY_FOLDER, PACK_FOLDER};
use super::pack::write_pack;
use super::volume::Volume;
use super::{BiomeTable, StateTable, section_volume};
use crate::chunk::{BlockDefaults, BlockState, Chunk, Section, is_invisible};
use crate::error::{Result, WorldmapError};
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

/// Writes the sections of the region at `region` into its pack under `structure_dir`, then removes the per-section files an
/// earlier render left for that region. Returns how many sections the pack holds.
pub fn write_region_pack(
    region: &Path,
    structure_dir: &Path,
    defaults: &BlockDefaults,
    states: &StateTable,
    biomes: &BiomeTable,
) -> Result<usize> {
    let stem = region
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| WorldmapError::Structure(format!("{} is not a region file", region.display())))?;
    let destination = structure_dir.join(PACK_FOLDER).join(format!("{stem}.obsr"));
    let written = write_pack(&destination, |pack| {
        for_each_chunk(region, defaults, |chunk| {
            for section in chunk.sections() {
                let origin = [chunk.x * 16, section.y() * 16, chunk.z * 16];
                let volume = section_volume(section, origin, states, biomes)?;
                if !is_open_air(section, &volume) {
                    pack.push([chunk.x, section.y(), chunk.z], &volume.encode())?;
                }
            }
            Ok(())
        })
    })?;
    let legacy = structure_dir.join(LEGACY_FOLDER).join(stem);
    if legacy.exists() {
        fs::remove_dir_all(legacy)?;
    }
    let _ = fs::remove_dir(structure_dir.join(LEGACY_FOLDER));
    Ok(written)
}

/// An all-air section with open-sky light reads back as the air a missing section already gives, so it is not stored.
fn is_open_air(section: &Section, volume: &Volume) -> bool {
    section.palette().iter().all(|state| is_invisible(&state.name))
        && (0..volume.voxels()).all(|index| volume.sky.at(index) == OPEN_SKY && volume.block.at(index) == 0)
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

#[cfg(test)]
mod tests {
    use super::super::pack::PackReader;
    use super::*;
    use crate::test_support::{chunk_nbt, chunk_nbt_with_sky_light, region_bytes};

    #[test]
    fn only_all_air_sections_with_open_sky_light_are_left_out() {
        let dir = tempfile::tempdir().unwrap();
        let region = dir.path().join("r.0.0.mca");
        let chunks = [
            (0, 0, chunk_nbt(0, 0, 3955, &[((1, 1, 1), "minecraft:stone")])),
            (1, 0, chunk_nbt(1, 0, 3955, &[])),
            (2, 0, chunk_nbt_with_sky_light(2, 0, 3955, 0)),
            (3, 0, chunk_nbt(3, 0, 3955, &[((1, 1, 1), "minecraft:cave_air")])),
        ];
        fs::write(&region, region_bytes(&chunks)).unwrap();
        let defaults = BlockDefaults::default();
        let mut states = StateTable::default();
        states.extend(vocabulary(&region, &defaults).unwrap().states);
        let structure = dir.path().join("structure");
        fs::create_dir_all(structure.join("sections").join("r.0.0")).unwrap();
        fs::write(structure.join("sections").join("r.0.0").join("section_0_0_0.obsv"), b"old").unwrap();
        let written = write_region_pack(&region, &structure, &defaults, &states, &BiomeTable::with_plains()).unwrap();
        let mut pack = PackReader::open(&structure.join("regions").join("r.0.0.obsr")).unwrap().unwrap();
        assert_eq!(written, 1);
        assert!(pack.section([0, 0, 0]).unwrap().is_some());
        assert!(pack.section([1, 0, 0]).unwrap().is_none());
        assert!(pack.section([2, 0, 0]).unwrap().is_none(), "stored light of 0 under open sky is raised to open sky");
        assert!(pack.section([3, 0, 0]).unwrap().is_none());
        assert!(!structure.join("sections").exists(), "the per-section files of the region are gone");
    }
}
