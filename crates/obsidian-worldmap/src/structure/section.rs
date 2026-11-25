//! Converts a parsed section into the volume record stored for it.

use super::tables::{BiomeTable, PLAINS_ID, StateTable};
use super::volume::{BIOME_CELLS, Biomes, Light, Volume};
use crate::chunk::{Light as ChunkLight, Section};
use crate::error::{Result, WorldmapError};

pub fn section_volume(section: &Section, origin: [i32; 3], states: &StateTable, biomes: &BiomeTable) -> Result<Volume> {
    let palette = section
        .palette()
        .iter()
        .map(|state| {
            states
                .id(state)
                .ok_or_else(|| WorldmapError::Structure(format!("{} is missing from the state table", state.name)))
        })
        .collect::<Result<Vec<u32>>>()?;
    Ok(Volume {
        origin,
        size: [16; 3],
        palette,
        indices: section.indices().to_vec(),
        sky: light(section.sky_light()),
        block: light(section.block_light()),
        biomes: Some(biome_cells(section, biomes)?),
    })
}

fn light(light: &ChunkLight) -> Light {
    match light {
        ChunkLight::Uniform(level) => Light::Uniform(*level),
        ChunkLight::Packed(bytes) => Light::Nibbles(bytes.to_vec()),
    }
}

fn biome_cells(section: &Section, biomes: &BiomeTable) -> Result<Biomes> {
    if section.biome_palette().is_empty() {
        return Ok(Biomes {
            palette: vec![PLAINS_ID],
            indices: vec![0; BIOME_CELLS],
        });
    }
    let palette = section
        .biome_palette()
        .iter()
        .map(|name| {
            biomes
                .id(name)
                .ok_or_else(|| WorldmapError::Structure(format!("{name} is missing from the biome table")))
        })
        .collect::<Result<Vec<u16>>>()?;
    let indices = if section.biome_indices().is_empty() {
        vec![0; BIOME_CELLS]
    } else {
        section.biome_indices().to_vec()
    };
    Ok(Biomes { palette, indices })
}
