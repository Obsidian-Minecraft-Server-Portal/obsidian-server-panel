//! The layout of a dimension's structure folder, and how a cell's bundle is read back from it. Each region keeps its sections
//! in one pack. A region that an earlier render wrote as one file per section is read that way until it is rendered again.

use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::cell::{SectionStore, assemble};
use super::pack::PackReader;
use crate::error::Result;
use crate::region::CHUNKS_PER_SIDE;

const STRUCTURE: &str = "structure";
pub(super) const PACK_FOLDER: &str = "regions";
pub(super) const LEGACY_FOLDER: &str = "sections";
const REGION_CHUNKS: i32 = CHUNKS_PER_SIDE as i32;

/// The region that holds a chunk, as region x and region z.
pub(super) fn region_of(chunk_x: i32, chunk_z: i32) -> (i32, i32) {
    (chunk_x.div_euclid(REGION_CHUNKS), chunk_z.div_euclid(REGION_CHUNKS))
}

/// Where a region's pack is kept, relative to its dimension folder.
pub fn region_pack_path((region_x, region_z): (i32, i32)) -> String {
    format!("{STRUCTURE}/{PACK_FOLDER}/r.{region_x}.{region_z}.obsr")
}

/// The bundle of one cell, read from the structure folder of its dimension.
pub fn cell_bundle(folder: &Path, cell: [i32; 3]) -> Result<Vec<u8>> {
    let mut store = RegionStore {
        folder: folder.to_path_buf(),
        packs: HashMap::new(),
    };
    assemble(cell, &mut store)
}

/// The sections of one bundle, each read from its region's pack. Each pack is opened once per bundle.
struct RegionStore {
    folder: PathBuf,
    packs: HashMap<(i32, i32), Option<PackReader>>,
}

impl SectionStore for RegionStore {
    fn section(&mut self, chunk_x: i32, section_y: i32, chunk_z: i32) -> Result<Option<Vec<u8>>> {
        let region = region_of(chunk_x, chunk_z);
        if !self.packs.contains_key(&region) {
            let pack = PackReader::open(&self.folder.join(region_pack_path(region)))?;
            self.packs.insert(region, pack);
        }
        if let Some(Some(pack)) = self.packs.get_mut(&region) {
            return pack.section([chunk_x, section_y, chunk_z]);
        }
        legacy_section(&self.folder, chunk_x, section_y, chunk_z)
    }
}

fn legacy_section(folder: &Path, chunk_x: i32, section_y: i32, chunk_z: i32) -> Result<Option<Vec<u8>>> {
    let (region_x, region_z) = region_of(chunk_x, chunk_z);
    let path = folder
        .join(STRUCTURE)
        .join(LEGACY_FOLDER)
        .join(format!("r.{region_x}.{region_z}"))
        .join(format!("section_{chunk_x}_{section_y}_{chunk_z}.obsv"));
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::cell::read_bundle;
    use super::super::pack::write_pack;
    use super::super::volume::{Light, Volume};
    use super::*;

    fn record() -> Vec<u8> {
        Volume {
            origin: [0, 0, 0],
            size: [16; 3],
            palette: vec![1],
            indices: vec![0; 4096],
            sky: Light::Uniform(15),
            block: Light::Uniform(0),
            biomes: None,
        }
        .encode()
    }

    #[test]
    fn a_pack_and_an_earlier_per_section_file_give_the_same_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let packed = dir.path().join("packed");
        let legacy = dir.path().join("legacy");
        let pack_path = packed.join(STRUCTURE).join(PACK_FOLDER).join("r.0.0.obsr");
        write_pack(&pack_path, |pack| pack.push([0, 0, 0], &record())).unwrap();
        let legacy_dir = legacy.join(STRUCTURE).join(LEGACY_FOLDER).join("r.0.0");
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::write(legacy_dir.join("section_0_0_0.obsv"), record()).unwrap();
        let from_pack = cell_bundle(&packed, [0, 0, 0]).unwrap();
        let from_legacy = cell_bundle(&legacy, [0, 0, 0]).unwrap();
        assert_eq!(from_pack, from_legacy);
        let (_, records) = read_bundle(&from_pack).unwrap();
        assert_eq!(records.len(), 1 + 26, "one own section and the 26 border volumes");
    }
}
