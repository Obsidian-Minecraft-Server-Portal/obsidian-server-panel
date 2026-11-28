//! Writes the structure a browser meshes for each pending region, along with the tables that give its ids meaning. Regions are
//! read on the map pool, whose threads start at the lowest priority the system offers, so a render never starves the server beside it.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use obsidian_common::Result;
use obsidian_worldmap::atlas::Atlas;
use obsidian_worldmap::atomic;
use obsidian_worldmap::render::Pack;
use obsidian_worldmap::structure::{BiomeTable, StateTable, lighting};
use rayon::prelude::*;
use serde::Deserialize;

use crate::background::pool;
use crate::job::Pending;
use crate::pacing::Pacer;

pub const STRUCTURE_FOLDER: &str = "structure";

/// A region the structure step could not read or write. It is left out of the batch and its pack is not replaced.
pub struct Skipped {
    pub key: String,
    pub source_modified: i64,
    pub reason: String,
}

impl Skipped {
    fn of(item: &Pending, reason: impl Display) -> Self {
        Self {
            key: item.key.clone(),
            source_modified: item.source_modified,
            reason: reason.to_string(),
        }
    }
}

/// Writes the structure of every pending region, grouped by dimension, and rewrites that dimension's tables. A region that
/// cannot be read or written is reported and the rest of its batch goes on.
pub fn write(pending: &[Pending], pack: &Pack, atlas: &Atlas, pacer: &Pacer) -> Result<Vec<Skipped>> {
    let mut skipped = Vec::new();
    let mut dimensions: BTreeMap<&Path, Vec<&Pending>> = BTreeMap::new();
    for item in pending {
        dimensions.entry(item.dimension_dir.as_path()).or_default().push(item);
    }
    for (dimension_dir, items) in dimensions {
        let directory = dimension_dir.join(STRUCTURE_FOLDER);
        fs::create_dir_all(&directory)?;
        let mut states = load_states(&directory)?;
        let mut biomes = load_biomes(&directory)?;
        let vocabularies = pool().install(|| {
            items
                .par_iter()
                .map(|item| {
                    pacer.wait_turn();
                    pack.region_vocabulary(&item.region_path)
                })
                .collect::<Vec<_>>()
        });
        let mut readable = Vec::with_capacity(items.len());
        for (item, vocabulary) in items.into_iter().zip(vocabularies) {
            match vocabulary {
                Ok(vocabulary) => {
                    states.extend(vocabulary.states);
                    biomes.extend(vocabulary.biomes);
                    readable.push(item);
                }
                Err(error) => skipped.push(Skipped::of(item, error)),
            }
        }
        atomic::write(&directory.join("states.json"), states.to_json().map_err(crate::job::other)?)?;
        let dimension_lighting = dimension_dir.file_name().and_then(|name| name.to_str()).map_or_else(|| lighting(""), lighting);
        atomic::write(&directory.join("blocks.json"), serde_json::to_vec(&pack.block_table(states.states(), atlas, &dimension_lighting))?)?;
        atomic::write(&directory.join("biomes.json"), serde_json::to_vec(&pack.biome_table(biomes.names()))?)?;
        let written = pool().install(|| {
            readable
                .par_iter()
                .map(|item| {
                    pacer.wait_turn();
                    pack.write_region_pack(&item.region_path, &directory, &states, &biomes)
                })
                .collect::<Vec<_>>()
        });
        for (item, result) in readable.into_iter().zip(written) {
            if let Err(error) = result {
                skipped.push(Skipped::of(item, error));
            }
        }
    }
    Ok(skipped)
}

fn load_states(directory: &Path) -> Result<StateTable> {
    match fs::read(directory.join("states.json")) {
        Ok(json) => StateTable::from_json(&json).map_err(crate::job::other),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(StateTable::default()),
        Err(error) => Err(error.into()),
    }
}

#[derive(Deserialize)]
struct StoredBiome {
    name: String,
}

fn load_biomes(directory: &Path) -> Result<BiomeTable> {
    let mut biomes = BiomeTable::with_plains();
    match fs::read(directory.join("biomes.json")) {
        Ok(json) => {
            let stored: Vec<StoredBiome> = serde_json::from_slice(&json)?;
            biomes.extend(stored.into_iter().map(|biome| biome.name));
            Ok(biomes)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(biomes),
        Err(error) => Err(error.into()),
    }
}
