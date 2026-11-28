//! Cushions are entities rather than blocks, so each one takes a voxel of the structure as a block state of its own.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use fastnbt::Value;
use serde::Deserialize;

use crate::chunk::BlockState;
use crate::models::CUSHION_STATE;
use crate::region::{CHUNKS_PER_SIDE, RegionFile};

const CUSHION_ENTITY: &str = "minecraft:cushion";
/// The horizontal facings in the order of the game's 2D direction data, a quarter turn apart from south.
const FACINGS: [&str; 4] = ["south", "west", "north", "east"];
/// A cushion's slab is a quarter block thick.
const SLAB_HEIGHT: f64 = 0.25;
/// Keeps a slab whose top lies exactly on a block boundary in the block below it.
const TOP_EPSILON: f64 = 1e-4;

/// The cushions of one region, keyed by the section that holds them. Each entry is a voxel index within that section, counted the way
/// the section's own indices are, with the state the cushion takes.
#[derive(Default)]
pub struct Overlays {
    sections: BTreeMap<[i32; 3], Vec<(usize, BlockState)>>,
}

impl Overlays {
    /// Reads the cushions of the region at `region`, from the entities folder beside its region folder. Entities are an overlay on the
    /// blocks, so an entity file or chunk that cannot be read leaves its cushions out rather than failing the region.
    pub fn read(region: &Path) -> Self {
        let mut overlays = Self::default();
        let Some(path) = entity_region(region).filter(|path| path.is_file()) else {
            return overlays;
        };
        let Ok(file) = RegionFile::open(&path) else { return overlays };
        for local_z in 0..CHUNKS_PER_SIDE {
            for local_x in 0..CHUNKS_PER_SIDE {
                let Ok(Some(nbt)) = file.chunk_nbt(local_x, local_z) else { continue };
                let Ok(chunk) = fastnbt::from_bytes::<EntityChunk>(&nbt) else { continue };
                for entity in &chunk.entities {
                    if let Some((position, state)) = cushion(entity) {
                        overlays.insert(position, state);
                    }
                }
            }
        }
        overlays
    }

    /// Every state an overlay writes, which the structure's state table must hold.
    pub fn states(&self) -> impl Iterator<Item = &BlockState> {
        self.sections.values().flatten().map(|(_, state)| state)
    }

    /// The overlays of the section at chunk x, section y and chunk z.
    pub fn in_section(&self, section: [i32; 3]) -> &[(usize, BlockState)] {
        self.sections.get(&section).map(Vec::as_slice).unwrap_or_default()
    }

    fn insert(&mut self, [x, y, z]: [i32; 3], state: BlockState) {
        let section = [x.div_euclid(16), y.div_euclid(16), z.div_euclid(16)];
        let [local_x, local_y, local_z] = [x, y, z].map(|coordinate| coordinate.rem_euclid(16) as usize);
        self.sections.entry(section).or_default().push(((local_y << 8) | (local_z << 4) | local_x, state));
    }
}

/// The entity file of a region: the entities folder beside the region's folder, under the same file name.
fn entity_region(region: &Path) -> Option<PathBuf> {
    Some(region.parent()?.parent()?.join("entities").join(region.file_name()?))
}

#[derive(Deserialize)]
struct EntityChunk {
    #[serde(rename = "Entities", default)]
    entities: Vec<Value>,
}

/// A cushion's voxel and the state it takes, or `None` for any other entity. The voxel is the one that holds the cushion's top. A
/// cushion on a short block keeps that block's voxel, so `offset` is the height of its bottom above the voxel's floor in sixteenths.
fn cushion(entity: &Value) -> Option<([i32; 3], BlockState)> {
    let Value::Compound(fields) = entity else { return None };
    match fields.get("id") {
        Some(Value::String(id)) if id == CUSHION_ENTITY => {}
        _ => return None,
    }
    let [x, y, z]: [f64; 3] = numbers(fields.get("Pos")?, double)?.try_into().ok()?;
    let yaw = numbers(fields.get("Rotation")?, float)?.first().copied()?;
    let colour = match fields.get("color") {
        Some(Value::String(colour)) => colour.clone(),
        _ => "white".to_owned(),
    };
    let facing = FACINGS[((yaw / 90.0 + 0.5).floor() as i32 & 3) as usize];
    let top_voxel = (y + SLAB_HEIGHT - TOP_EPSILON).floor();
    let offset = ((y - top_voxel) * 16.0).round() as i32;
    let state = BlockState {
        name: CUSHION_STATE.to_owned(),
        properties: BTreeMap::from([
            ("color".to_owned(), colour),
            ("facing".to_owned(), facing.to_owned()),
            ("offset".to_owned(), offset.to_string()),
        ]),
    };
    Some(([x.floor() as i32, top_voxel as i32, z.floor() as i32], state))
}

fn numbers<T>(value: &Value, pick: fn(&Value) -> Option<T>) -> Option<Vec<T>> {
    let Value::List(items) = value else { return None };
    items.iter().map(pick).collect()
}

fn double(value: &Value) -> Option<f64> {
    match value {
        Value::Double(number) => Some(*number),
        _ => None,
    }
}

fn float(value: &Value) -> Option<f32> {
    match value {
        Value::Float(number) => Some(*number),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde::Serialize;

    use super::*;
    use crate::test_support::region_bytes;

    #[derive(Serialize)]
    struct Entities {
        #[serde(rename = "Entities")]
        entities: Vec<Entity>,
    }

    #[derive(Serialize)]
    struct Entity {
        id: &'static str,
        #[serde(rename = "Pos")]
        pos: Vec<f64>,
        #[serde(rename = "Rotation")]
        rotation: Vec<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<&'static str>,
    }

    fn entity(id: &'static str, pos: [f64; 3], yaw: f32, color: Option<&'static str>) -> Entity {
        Entity {
            id,
            pos: pos.to_vec(),
            rotation: vec![yaw, 0.0],
            color,
        }
    }

    fn cushion_state(colour: &str, facing: &str, offset: &str) -> BlockState {
        BlockState {
            name: CUSHION_STATE.into(),
            properties: BTreeMap::from([
                ("color".to_owned(), colour.to_owned()),
                ("facing".to_owned(), facing.to_owned()),
                ("offset".to_owned(), offset.to_owned()),
            ]),
        }
    }

    #[test]
    fn cushions_take_the_voxel_holding_their_top_and_other_entities_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let region = dir.path().join("region").join("r.0.0.mca");
        fs::create_dir_all(dir.path().join("entities")).unwrap();
        let chunk = Entities {
            entities: vec![
                entity("minecraft:cushion", [1.5, 0.0, 1.5], 90.0, Some("red")),
                entity("minecraft:zombie", [3.5, 64.0, 3.5], 0.0, None),
                entity("minecraft:cushion", [33.5, 0.0, 1.5], 0.0, None),
                entity("minecraft:cushion", [5.5, 80.9375, 5.5], 0.0, None),
            ],
        };
        let nbt = fastnbt::to_bytes(&chunk).unwrap();
        fs::write(dir.path().join("entities").join("r.0.0.mca"), region_bytes(&[(0, 0, nbt)])).unwrap();
        let overlays = Overlays::read(&region);
        assert_eq!(overlays.in_section([0, 0, 0]).to_vec(), vec![(17, cushion_state("red", "west", "0"))]);
        assert_eq!(overlays.in_section([2, 0, 0]).to_vec(), vec![(17, cushion_state("white", "south", "0"))]);
        // On a dirt path, which is 15 sixteenths high, the cushion's top is in the voxel above the path, a sixteenth below its floor.
        assert_eq!(overlays.in_section([0, 5, 0]).to_vec(), vec![(341, cushion_state("white", "south", "-1"))]);
        assert!(overlays.in_section([0, 4, 0]).is_empty());
    }

    #[test]
    fn an_empty_entity_file_holds_no_cushions() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("entities")).unwrap();
        fs::write(dir.path().join("entities").join("r.0.0.mca"), []).unwrap();
        assert!(Overlays::read(&dir.path().join("region").join("r.0.0.mca")).in_section([0, 0, 0]).is_empty());
    }
}
