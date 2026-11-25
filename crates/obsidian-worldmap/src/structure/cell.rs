//! Cell bundles: everything a browser needs to mesh one cell of 64 by 64 by 64 blocks, in one response. The cell's own
//! sections come first, then 26 border volumes with the blocks and light of the cells around it, so the mesher needs no
//! further requests. Little-endian. Layout: magic `OBSB`, version, reserved, cell x, y, z as i32, volume count as u32, then
//! each volume as a length and its `OBSV` record.

use std::collections::HashMap;

use super::tables::AIR_ID;
use super::volume::{Light, Volume};
use crate::error::{Result, WorldmapError};

pub const CELL_BLOCKS: i32 = 64;
const SECTION_BLOCKS: i32 = 16;
const CHUNKS_PER_CELL: i32 = 4;
const SECTIONS_PER_CELL: i32 = 4;
const MAGIC: [u8; 4] = *b"OBSB";
const VERSION: u16 = 1;
pub(super) const OPEN_SKY: u8 = 15;

/// The stored volume record of the section at a chunk and section coordinate, or `None` when none is stored.
pub trait SectionStore {
    fn section(&mut self, chunk_x: i32, section_y: i32, chunk_z: i32) -> Result<Option<Vec<u8>>>;
}

/// Bundles the cell at `cell` (in units of 64 blocks) from the sections in `store`.
pub fn assemble<S: SectionStore>(cell: [i32; 3], store: &mut S) -> Result<Vec<u8>> {
    let mut volumes = Vec::new();
    for section_y in cell[1] * SECTIONS_PER_CELL..(cell[1] + 1) * SECTIONS_PER_CELL {
        for chunk_z in cell[2] * CHUNKS_PER_CELL..(cell[2] + 1) * CHUNKS_PER_CELL {
            for chunk_x in cell[0] * CHUNKS_PER_CELL..(cell[0] + 1) * CHUNKS_PER_CELL {
                if let Some(record) = store.section(chunk_x, section_y, chunk_z)? {
                    volumes.push(record);
                }
            }
        }
    }
    let origin = cell.map(|coordinate| coordinate * CELL_BLOCKS);
    let mut neighbours = Neighbours::new(store);
    for offset in shell_offsets() {
        volumes.push(border_volume(origin, offset, &mut neighbours)?.encode());
    }
    Ok(bundle(cell, &volumes))
}

/// The cell and the records it holds.
pub fn read_bundle(bytes: &[u8]) -> Result<([i32; 3], Vec<Vec<u8>>)> {
    let invalid = || WorldmapError::Structure("cell bundle is malformed".into());
    if bytes.len() < 24 || bytes[..4] != MAGIC {
        return Err(invalid());
    }
    let word = |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let cell = [word(8) as i32, word(12) as i32, word(16) as i32];
    let count = word(20) as usize;
    let mut records = Vec::with_capacity(count);
    let mut at = 24;
    for _ in 0..count {
        let length = word(at) as usize;
        let record = bytes.get(at + 4..at + 4 + length).ok_or_else(invalid)?;
        records.push(record.to_vec());
        at += 4 + length;
    }
    Ok((cell, records))
}

fn bundle(cell: [i32; 3], volumes: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    for coordinate in cell {
        out.extend_from_slice(&coordinate.to_le_bytes());
    }
    out.extend_from_slice(&(volumes.len() as u32).to_le_bytes());
    for volume in volumes {
        out.extend_from_slice(&(volume.len() as u32).to_le_bytes());
        out.extend_from_slice(volume);
    }
    out
}

fn shell_offsets() -> impl Iterator<Item = [i32; 3]> {
    (-1..=1).flat_map(|x| (-1..=1).flat_map(move |y| (-1..=1).map(move |z| [x, y, z])))
        .filter(|offset| *offset != [0, 0, 0])
}

fn border_volume<S: SectionStore>(origin: [i32; 3], offset: [i32; 3], neighbours: &mut Neighbours<'_, S>) -> Result<Volume> {
    let min = std::array::from_fn(|axis| match offset[axis] {
        -1 => origin[axis] - 1,
        0 => origin[axis],
        _ => origin[axis] + CELL_BLOCKS,
    });
    let size = std::array::from_fn(|axis| if offset[axis] == 0 { CELL_BLOCKS as u8 } else { 1 });
    let voxels: usize = size.iter().map(|&side| usize::from(side)).product();
    let mut block_ids = Vec::with_capacity(voxels);
    let mut sky = Vec::with_capacity(voxels);
    let mut block = Vec::with_capacity(voxels);
    for y in 0..i32::from(size[1]) {
        for z in 0..i32::from(size[2]) {
            for x in 0..i32::from(size[0]) {
                let sample = neighbours.sample([min[0] + x, min[1] + y, min[2] + z])?;
                block_ids.push(sample.id);
                sky.push(sample.sky);
                block.push(sample.block);
            }
        }
    }
    let mut palette: Vec<u32> = Vec::new();
    let indices = block_ids
        .iter()
        .map(|&id| {
            palette.iter().position(|&known| known == id).unwrap_or_else(|| {
                palette.push(id);
                palette.len() - 1
            }) as u16
        })
        .collect();
    Ok(Volume {
        origin: min,
        size,
        palette,
        indices,
        sky: Light::from_levels(&sky),
        block: Light::from_levels(&block),
        biomes: None,
    })
}

#[derive(Clone, Copy)]
struct Sample {
    id: u32,
    sky: u8,
    block: u8,
}

const ABSENT: Sample = Sample {
    id: AIR_ID,
    sky: OPEN_SKY,
    block: 0,
};

struct Neighbours<'a, S: SectionStore> {
    store: &'a mut S,
    decoded: HashMap<[i32; 3], Option<Volume>>,
}

impl<'a, S: SectionStore> Neighbours<'a, S> {
    fn new(store: &'a mut S) -> Self {
        Self {
            store,
            decoded: HashMap::new(),
        }
    }

    fn sample(&mut self, world: [i32; 3]) -> Result<Sample> {
        let section = [
            world[0].div_euclid(SECTION_BLOCKS),
            world[1].div_euclid(SECTION_BLOCKS),
            world[2].div_euclid(SECTION_BLOCKS),
        ];
        if !self.decoded.contains_key(&section) {
            let record = self.store.section(section[0], section[1], section[2])?;
            let volume = record.map(|bytes| Volume::decode(&bytes)).transpose()?;
            self.decoded.insert(section, volume);
        }
        let Some(volume) = self.decoded.get(&section).and_then(Option::as_ref) else {
            return Ok(ABSENT);
        };
        let local = world.map(|coordinate| coordinate.rem_euclid(SECTION_BLOCKS) as usize);
        let index = local[0] + 16 * local[2] + 256 * local[1];
        Ok(Sample {
            id: volume.palette[usize::from(volume.indices[index])],
            sky: volume.sky.at(index),
            block: volume.block.at(index),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::volume::Light as VoxelLight;

    struct World {
        sections: HashMap<[i32; 3], Vec<u8>>,
    }

    impl SectionStore for World {
        fn section(&mut self, chunk_x: i32, section_y: i32, chunk_z: i32) -> Result<Option<Vec<u8>>> {
            Ok(self.sections.get(&[chunk_x, section_y, chunk_z]).cloned())
        }
    }

    fn voxel(world: [i32; 3]) -> (u32, u8, u8) {
        let id = if (world[0] + world[1] * 3 + world[2]).rem_euclid(5) == 0 { 2 } else { 0 };
        (id, (world[0].rem_euclid(16) as u8) % 16, (world[2].rem_euclid(16) as u8) % 16)
    }

    fn section_record(chunk_x: i32, section_y: i32, chunk_z: i32) -> Vec<u8> {
        let origin = [chunk_x * 16, section_y * 16, chunk_z * 16];
        let mut ids = Vec::new();
        let mut sky = Vec::new();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let (id, s, _) = voxel([origin[0] + x, origin[1] + y, origin[2] + z]);
                    ids.push(id);
                    sky.push(s);
                }
            }
        }
        let mut palette = vec![0_u32];
        let mut indices = Vec::new();
        for id in ids {
            let index = palette.iter().position(|&known| known == id).unwrap_or_else(|| {
                palette.push(id);
                palette.len() - 1
            });
            indices.push(index as u16);
        }
        Volume {
            origin,
            size: [16; 3],
            palette,
            indices,
            sky: VoxelLight::from_levels(&sky),
            block: VoxelLight::Uniform(0),
            biomes: None,
        }
        .encode()
    }

    #[test]
    fn border_volumes_match_the_blocks_and_light_of_the_neighbouring_cells() {
        let cell = [0, 0, 0];
        let mut sections = HashMap::new();
        for chunk_z in -1..=4 {
            for chunk_x in -1..=4 {
                for section_y in -1..=4 {
                    sections.insert([chunk_x, section_y, chunk_z], section_record(chunk_x, section_y, chunk_z));
                }
            }
        }
        let bundle_bytes = assemble(cell, &mut World { sections }).unwrap();
        let (read_cell, records) = read_bundle(&bundle_bytes).unwrap();
        assert_eq!(read_cell, cell);
        let volumes: Vec<Volume> = records.iter().map(|record| Volume::decode(record).unwrap()).collect();
        let mut checked = 0;
        for volume in &volumes {
            for index in 0..volume.voxels() {
                let x = index % usize::from(volume.size[0]);
                let z = index / usize::from(volume.size[0]) % usize::from(volume.size[2]);
                let y = index / (usize::from(volume.size[0]) * usize::from(volume.size[2]));
                let world = [
                    volume.origin[0] + x as i32,
                    volume.origin[1] + y as i32,
                    volume.origin[2] + z as i32,
                ];
                let (expected_id, expected_sky, _) = voxel(world);
                assert_eq!(volume.palette[usize::from(volume.indices[index])], expected_id, "block at {world:?}");
                assert_eq!(volume.sky.at(index), expected_sky, "sky at {world:?}");
                checked += 1;
            }
        }
        assert_eq!(checked, 66 * 66 * 66);
    }

    fn air_record(chunk_x: i32, section_y: i32, chunk_z: i32) -> Vec<u8> {
        Volume {
            origin: [chunk_x * 16, section_y * 16, chunk_z * 16],
            size: [16; 3],
            palette: vec![AIR_ID],
            indices: vec![0; 4096],
            sky: VoxelLight::Uniform(OPEN_SKY),
            block: VoxelLight::Uniform(0),
            biomes: None,
        }
        .encode()
    }

    fn voxel_values(bundle: &[u8]) -> HashMap<[i32; 3], (u32, u8, u8)> {
        let (_, records) = read_bundle(bundle).unwrap();
        let mut values = HashMap::new();
        for record in records {
            let volume = Volume::decode(&record).unwrap();
            for index in 0..volume.voxels() {
                let x = index % usize::from(volume.size[0]);
                let z = index / usize::from(volume.size[0]) % usize::from(volume.size[2]);
                let y = index / (usize::from(volume.size[0]) * usize::from(volume.size[2]));
                let world = [volume.origin[0] + x as i32, volume.origin[1] + y as i32, volume.origin[2] + z as i32];
                let id = volume.palette[usize::from(volume.indices[index])];
                values.insert(world, (id, volume.sky.at(index), volume.block.at(index)));
            }
        }
        values
    }

    #[test]
    fn a_missing_open_sky_air_section_reads_like_a_stored_one() {
        let cell = [0, 0, 0];
        let mut stored = HashMap::new();
        let mut missing = HashMap::new();
        for chunk_z in -1..=4 {
            for chunk_x in -1..=4 {
                for section_y in -1..=4 {
                    let key = [chunk_x, section_y, chunk_z];
                    if section_y == 2 {
                        stored.insert(key, air_record(chunk_x, section_y, chunk_z));
                    } else {
                        let record = section_record(chunk_x, section_y, chunk_z);
                        stored.insert(key, record.clone());
                        missing.insert(key, record);
                    }
                }
            }
        }
        let with_air = voxel_values(&assemble(cell, &mut World { sections: stored }).unwrap());
        let without_air = voxel_values(&assemble(cell, &mut World { sections: missing }).unwrap());
        let open_air = (AIR_ID, OPEN_SKY, 0);
        for x in -1..=CELL_BLOCKS {
            for y in -1..=CELL_BLOCKS {
                for z in -1..=CELL_BLOCKS {
                    let world = [x, y, z];
                    let expected = with_air.get(&world).copied().unwrap_or(open_air);
                    assert_eq!(without_air.get(&world).copied().unwrap_or(open_air), expected, "at {world:?}");
                }
            }
        }
    }
}
