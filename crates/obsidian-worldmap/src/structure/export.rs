//! The tables a browser needs to mesh a dimension: each state's baked faces and flags, and each biome's tint colours.

use std::collections::HashMap;

use serde::Serialize;

use crate::assets::AssetStore;
use crate::atlas::{Atlas, Layer};
use crate::biome::{Biome, BiomeRegistry, GrassModifier};
use crate::chunk::{BlockState, is_invisible};
use crate::colour::Tints;
use crate::models::bake_state;
use crate::models::geometry::{Direction, Quad, TintKind};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTable {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub names: Vec<String>,
    pub states: Vec<BlockEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEntry {
    /// Index into the table's names.
    pub name: u32,
    pub air: bool,
    /// 0 for none, 1 for water, 2 for lava.
    pub fluid: u8,
    /// Bit `n` is set when the state has a full opaque face on the side with direction index `n`.
    pub full: u8,
    pub quads: Vec<QuadEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuadEntry {
    /// Four corners as x, y, z in model units, 0 to 16.
    pub corners: [f32; 12],
    /// Texel coordinates within the texture's 16 by 16 cell, two per corner.
    pub uvs: [f32; 8],
    /// Top left texel of the texture's cell in the atlas.
    pub cell: [u32; 2],
    pub direction: u8,
    /// Direction index of the cullface, or -1.
    pub cullface: i8,
    /// 0 for grass, 1 for foliage, 2 for water, or -1 for no tint.
    pub tint: i8,
    /// 0 for opaque, 1 for cutout, 2 for translucent.
    pub layer: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiomeEntry {
    pub name: String,
    /// Grass and foliage before any position-dependent modifier.
    pub grass: [u8; 3],
    pub foliage: [u8; 3],
    pub water: [u8; 3],
    pub grass_modifier: &'static str,
}

pub fn block_table(assets: &AssetStore, states: &[BlockState], atlas: &Atlas) -> BlockTable {
    let mut names: Vec<String> = Vec::new();
    let mut name_ids: HashMap<&str, u32> = HashMap::new();
    let entries = states
        .iter()
        .map(|state| {
            let name = *name_ids.entry(state.name.as_str()).or_insert_with(|| {
                names.push(state.name.clone());
                (names.len() - 1) as u32
            });
            let quads = bake_state(assets, state);
            BlockEntry {
                name,
                air: is_invisible(&state.name),
                fluid: fluid_kind(&state.name),
                full: full_mask(&quads, atlas),
                quads: quads.iter().map(|quad| quad_entry(quad, atlas)).collect(),
            }
        })
        .collect();
    let (atlas_width, atlas_height) = atlas.image().dimensions();
    BlockTable {
        atlas_width,
        atlas_height,
        names,
        states: entries,
    }
}

pub fn biome_table(assets: &AssetStore, biomes: &BiomeRegistry, names: &[String]) -> Vec<BiomeEntry> {
    let tints = Tints::new(assets);
    names
        .iter()
        .map(|name| {
            let biome: &Biome = biomes.get(name);
            BiomeEntry {
                name: name.clone(),
                grass: tints.base_colour(TintKind::Grass, biome),
                foliage: tints.base_colour(TintKind::Foliage, biome),
                water: tints.base_colour(TintKind::Water, biome),
                grass_modifier: modifier_name(biome.grass_modifier),
            }
        })
        .collect()
}

fn fluid_kind(name: &str) -> u8 {
    match name {
        "minecraft:water" => 1,
        "minecraft:lava" => 2,
        _ => 0,
    }
}

fn full_mask(quads: &[Quad], atlas: &Atlas) -> u8 {
    full_faces(quads, atlas)
        .iter()
        .enumerate()
        .filter(|(_, full)| **full)
        .fold(0, |mask, (index, _)| mask | (1 << index))
}

/// The sides a block fills with an opaque face, indexed by direction: a quad lying on a side and spanning its whole face.
fn full_faces(quads: &[Quad], atlas: &Atlas) -> [bool; 6] {
    let mut full = [false; 6];
    for quad in quads {
        if atlas.layer(&quad.texture) == Layer::Opaque && covers_block_face(quad) {
            full[usize::from(quad.direction.index())] = true;
        }
    }
    full
}

fn covers_block_face(quad: &Quad) -> bool {
    let (axis, boundary) = match quad.direction {
        Direction::Down => (1, 0.0),
        Direction::Up => (1, 16.0),
        Direction::North => (2, 0.0),
        Direction::South => (2, 16.0),
        Direction::West => (0, 0.0),
        Direction::East => (0, 16.0),
    };
    if quad.corners.iter().any(|corner| (corner[axis] - boundary).abs() > 1e-3) {
        return false;
    }
    [0, 1, 2].into_iter().filter(|other| *other != axis).all(|other| {
        let low = quad.corners.iter().map(|corner| corner[other]).fold(f32::MAX, f32::min);
        let high = quad.corners.iter().map(|corner| corner[other]).fold(f32::MIN, f32::max);
        low.abs() < 1e-3 && (high - 16.0).abs() < 1e-3
    })
}

fn quad_entry(quad: &Quad, atlas: &Atlas) -> QuadEntry {
    QuadEntry {
        corners: quad.corners.as_flattened().try_into().expect("four corners of three coordinates"),
        uvs: quad.uvs.as_flattened().try_into().expect("four corners of two coordinates"),
        cell: atlas.cell_origin(&quad.texture),
        direction: quad.direction.index(),
        cullface: quad.cullface.map_or(-1, |face: Direction| face.index() as i8),
        tint: match quad.tint {
            Some(TintKind::Grass) => 0,
            Some(TintKind::Foliage) => 1,
            Some(TintKind::Water) => 2,
            None => -1,
        },
        layer: match atlas.layer(&quad.texture) {
            Layer::Opaque => 0,
            Layer::Cutout => 1,
            Layer::Translucent => 2,
        },
    }
}

fn modifier_name(modifier: GrassModifier) -> &'static str {
    match modifier {
        GrassModifier::None => "none",
        GrassModifier::DarkForest => "darkForest",
        GrassModifier::Swamp => "swamp",
    }
}
