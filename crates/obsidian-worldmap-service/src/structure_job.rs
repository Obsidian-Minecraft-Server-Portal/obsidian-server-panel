//! Writes the structure a browser meshes for each pending region, along with the tables that give its ids meaning. Regions are
//! read on the map pool, whose threads start at the lowest priority the system offers, so a render never starves the server beside it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use obsidian_common::Result;
use obsidian_worldmap::atlas::Atlas;
use obsidian_worldmap::render::Pack;
use obsidian_worldmap::structure::{BiomeTable, StateTable, ambient_light};
use rayon::prelude::*;
use serde::Deserialize;

use crate::background::pool;
use crate::job::{Pending, other};
use crate::pacing::Pacer;

pub const STRUCTURE_FOLDER: &str = "structure";

/// Writes the structure of every pending region, grouped by dimension, and rewrites that dimension's tables.
pub fn write(pending: &[Pending], pack: &Pack, atlas: &Atlas, pacer: &Pacer) -> Result<()> {
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
                    pack.region_vocabulary(&item.region_path).map_err(other)
                })
                .collect::<Result<Vec<_>>>()
        })?;
        let mut block_states = BTreeSet::new();
        let mut biome_names = BTreeSet::new();
        for vocabulary in vocabularies {
            block_states.extend(vocabulary.states);
            biome_names.extend(vocabulary.biomes);
        }
        states.extend(block_states);
        biomes.extend(biome_names);
        fs::write(directory.join("states.json"), states.to_json().map_err(other)?)?;
        let ambient = dimension_dir.file_name().and_then(|name| name.to_str()).map_or(0.0, ambient_light);
        fs::write(directory.join("blocks.json"), serde_json::to_vec(&pack.block_table(states.states(), atlas, ambient))?)?;
        fs::write(directory.join("biomes.json"), serde_json::to_vec(&pack.biome_table(biomes.names()))?)?;
        pool().install(|| {
            items.par_iter().try_for_each(|item| {
                pacer.wait_turn();
                pack.write_region_pack(&item.region_path, &directory, &states, &biomes)
                    .map(|_| ())
                    .map_err(other)
            })
        })?;
    }
    Ok(())
}

fn load_states(directory: &Path) -> Result<StateTable> {
    match fs::read(directory.join("states.json")) {
        Ok(json) => StateTable::from_json(&json).map_err(other),
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
