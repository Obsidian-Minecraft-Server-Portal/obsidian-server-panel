//! Converts a parsed section into the volume record stored for it.

use super::tables::{BiomeTable, PLAINS_ID, StateTable};
use super::volume::{BIOME_CELLS, Biomes, Light, Volume};
use crate::chunk::{BlockState, Light as ChunkLight, Section};
use crate::error::{Result, WorldmapError};

/// Converts a section into its volume. Each overlay replaces the voxel at its index with the state it names, which the section's own
/// palette need not hold.
pub fn section_volume(
    section: &Section,
    origin: [i32; 3],
    states: &StateTable,
    biomes: &BiomeTable,
    overlays: &[(usize, BlockState)],
) -> Result<Volume> {
    let mut palette = section.palette().iter().map(|state| state_id(states, state)).collect::<Result<Vec<u32>>>()?;
    let mut indices = section.indices().to_vec();
    for (index, state) in overlays {
        let id = state_id(states, state)?;
        let slot = match palette.iter().position(|&existing| existing == id) {
            Some(slot) => slot,
            None => {
                palette.push(id);
                palette.len() - 1
            }
        };
        indices[*index] = slot as u16;
    }
    Ok(Volume {
        origin,
        size: [16; 3],
        palette,
        indices,
        sky: light(section.sky_light()),
        block: light(section.block_light()),
        biomes: Some(biome_cells(section, biomes)?),
    })
}

fn state_id(states: &StateTable, state: &BlockState) -> Result<u32> {
    states
        .id(state)
        .ok_or_else(|| WorldmapError::Structure(format!("{} is missing from the state table", state.name)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;
    use crate::test_support::chunk_nbt;

    #[test]
    fn overlays_replace_the_voxels_they_name() {
        let chunk = Chunk::parse(&chunk_nbt(0, 0, 3955, &[((2, 0, 2), "minecraft:stone")])).unwrap();
        let section = &chunk.sections()[0];
        let cushion = BlockState {
            name: "minecraft:cushion".into(),
            properties: Default::default(),
        };
        let mut states = StateTable::default();
        states.extend(section.palette().iter().cloned());
        states.extend([cushion.clone()]);
        let volume = section_volume(section, [0, 0, 0], &states, &BiomeTable::with_plains(), &[(17, cushion.clone())]).unwrap();
        assert_eq!(volume.palette[usize::from(volume.indices[17])], states.id(&cushion).unwrap());
        let stone = section.palette().iter().find(|state| state.name == "minecraft:stone").unwrap();
        assert_eq!(volume.palette[usize::from(volume.indices[34])], states.id(stone).unwrap());
    }
}
