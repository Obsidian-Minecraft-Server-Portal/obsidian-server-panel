//! The tables a browser needs to mesh a dimension: each state's baked faces and flags, and each biome's tint colours.

use std::collections::HashMap;

use serde::Serialize;

use crate::assets::AssetStore;
use crate::atlas::{Atlas, Layer};
use crate::biome::{Biome, BiomeRegistry, GrassModifier};
use crate::chunk::{BlockState, is_invisible};
use crate::colour::Tints;
use crate::models::geometry::{BlockTint, Direction, Quad, TintKind};
use crate::models::{bake_state, holds_water};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTable {
    pub atlas_width: u32,
    pub atlas_height: u32,
    /// The least light a block takes in its dimension, from 0 to 1.
    pub ambient_light: f32,
    /// The brightness of a face by direction (down, up, north, south, west, east), as the dimension's type sets it.
    pub cardinal_lighting: [f32; 6],
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
    /// A fluid state's level: 0 is a source, 1 to 7 flow thinner, and 8 and up are falling.
    pub fluid_level: u8,
    /// Bit `n` is set when the state has a full opaque face on the side with direction index `n`.
    pub full: u8,
    /// Whether the state's collision is a full block. Faces flat against one take the game's cube lighting.
    pub collision_full: bool,
    /// Whether the state darkens the corners of the blocks beside it: a full collision, except for transparent blocks.
    pub shade_full: bool,
    /// Which faces the state leaves out beside a neighbour of the same block: 0 for none, 1 for the same block.
    pub skip: u8,
    /// Whether the state is iron bars or a pane, which leave out the faces that meet a neighbouring bar.
    pub bars: bool,
    /// Bit `n` is set when the state connects toward the side with direction index `n`, for bars and panes.
    pub connections: u8,
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
    /// Direction index of the shade, or -1 for full brightness.
    pub shade: i8,
    /// 0 for grass, 1 for foliage, 2 for water, 3 for dry foliage, or -1 for no biome tint.
    pub tint: i8,
    /// The fixed colour the block state carries, when it has one.
    pub colour: Option<[u8; 3]>,
    /// Whether the face takes ambient occlusion from its neighbours.
    pub ao: bool,
    /// Whether the face is part of a fluid's surface, which follows the fluid's height.
    pub fluid: bool,
    /// 0 for opaque, 1 for cutout, 2 for translucent.
    pub layer: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiomeEntry {
    pub name: String,
    /// Grass, foliage and dry foliage before any position-dependent modifier.
    pub grass: [u8; 3],
    pub foliage: [u8; 3],
    pub dry_foliage: [u8; 3],
    pub water: [u8; 3],
    pub grass_modifier: &'static str,
}

/// The light a dimension's type gives its blocks.
pub struct Lighting {
    pub ambient: f32,
    pub cardinal: [f32; 6],
}

/// The overworld and the end use the default face brightness. The nether's down and up faces are brighter.
pub fn lighting(dimension: &str) -> Lighting {
    match dimension {
        "minecraft_the_nether" => Lighting {
            ambient: 0.1,
            cardinal: [0.9, 0.9, 0.8, 0.8, 0.6, 0.6],
        },
        "minecraft_the_end" => Lighting {
            ambient: 0.25,
            cardinal: [0.5, 1.0, 0.8, 0.8, 0.6, 0.6],
        },
        _ => Lighting {
            ambient: 0.0,
            cardinal: [0.5, 1.0, 0.8, 0.8, 0.6, 0.6],
        },
    }
}

pub fn block_table(assets: &AssetStore, states: &[BlockState], atlas: &Atlas, lighting: &Lighting) -> BlockTable {
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
            let collision_full = full_collision(&quads);
            BlockEntry {
                name,
                air: is_invisible(&state.name),
                fluid: fluid_kind(state),
                fluid_level: fluid_level(state),
                full: if does_not_occlude(&state.name) { 0 } else { full_mask(&quads, atlas) },
                collision_full,
                shade_full: collision_full && !is_transparent(&state.name),
                skip: skip_kind(&state.name),
                bars: is_bars(&state.name),
                connections: connections(state),
                quads: quads.iter().map(|quad| quad_entry(quad, atlas)).collect(),
            }
        })
        .collect();
    let (atlas_width, atlas_height) = atlas.image().dimensions();
    BlockTable {
        atlas_width,
        atlas_height,
        ambient_light: lighting.ambient,
        cardinal_lighting: lighting.cardinal,
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
                dry_foliage: tints.base_colour(TintKind::DryFoliage, biome),
                water: tints.base_colour(TintKind::Water, biome),
                grass_modifier: modifier_name(biome.grass_modifier),
            }
        })
        .collect()
}

fn fluid_kind(state: &BlockState) -> u8 {
    match state.name.as_str() {
        "minecraft:water" => 1,
        "minecraft:lava" => 2,
        _ if holds_water(state) => 1,
        _ => 0,
    }
}

fn fluid_level(state: &BlockState) -> u8 {
    match state.name.as_str() {
        "minecraft:water" | "minecraft:lava" => state.properties.get("level").and_then(|level| level.parse().ok()).unwrap_or(0),
        _ => 0,
    }
}

fn skip_kind(name: &str) -> u8 {
    u8::from(is_half_transparent(name))
}

/// Blocks the game draws as a `TransparentBlock`: they shade nothing beside them.
fn is_transparent(name: &str) -> bool {
    matches!(name, "minecraft:glass" | "minecraft:tinted_glass") || name.ends_with("_stained_glass") || name.ends_with("copper_grate")
}

/// Blocks the game draws as a `HalfTransparentBlock` or one of its subclasses: faces against the same block are left out.
fn is_half_transparent(name: &str) -> bool {
    is_transparent(name) || matches!(name, "minecraft:ice" | "minecraft:frosted_ice" | "minecraft:honey_block" | "minecraft:slime_block")
}

/// Blocks the game gives `noOcclusion`, so the faces beside them are never culled by them: leaves, shulker boxes, translucent
/// blocks and bars.
fn does_not_occlude(name: &str) -> bool {
    name.ends_with("_leaves") || name.ends_with("shulker_box") || is_half_transparent(name) || is_bars(name)
}

fn is_bars(name: &str) -> bool {
    name.ends_with("_bars") || name.ends_with("glass_pane")
}

fn connections(state: &BlockState) -> u8 {
    [("north", 2), ("south", 3), ("west", 4), ("east", 5)]
        .into_iter()
        .filter(|(property, _)| state.properties.get(*property).is_some_and(|value| value == "true"))
        .fold(0, |mask, (_, face)| mask | (1 << face))
}

/// Fluid cubes do not count: water has no collision shape, so it leaves the corners beside it bright.
fn full_collision(quads: &[Quad]) -> bool {
    Direction::ALL
        .iter()
        .all(|&direction| quads.iter().any(|quad| !quad.fluid && quad.direction == direction && covers_block_face(quad)))
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
        shade: quad.shade.map_or(-1, |face: Direction| face.index() as i8),
        tint: match quad.tint {
            Some(BlockTint::Biome(TintKind::Grass)) => 0,
            Some(BlockTint::Biome(TintKind::Foliage)) => 1,
            Some(BlockTint::Biome(TintKind::Water)) => 2,
            Some(BlockTint::Biome(TintKind::DryFoliage)) => 3,
            Some(BlockTint::Fixed(_)) | None => -1,
        },
        colour: match quad.tint {
            Some(BlockTint::Fixed(colour)) => Some(colour),
            _ => None,
        },
        ao: quad.ambient_occlusion,
        fluid: quad.fluid,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn state(name: &str, properties: &[(&str, &str)]) -> BlockState {
        BlockState {
            name: name.into(),
            properties: properties.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn leaves_neither_occlude_nor_skip_their_neighbours() {
        assert!(does_not_occlude("minecraft:oak_leaves"));
        assert_eq!(skip_kind("minecraft:oak_leaves"), 0);
    }

    #[test]
    fn transparent_blocks_skip_their_own_kind_and_shade_nothing() {
        assert!(is_transparent("minecraft:white_stained_glass"));
        assert_eq!(skip_kind("minecraft:glass"), 1);
        assert!(is_half_transparent("minecraft:ice") && !is_transparent("minecraft:ice"));
        assert_eq!(skip_kind("minecraft:ice"), 1);
    }

    #[test]
    fn bars_connect_toward_their_true_properties() {
        let pane = state("minecraft:glass_pane", &[("north", "true"), ("south", "false"), ("east", "true"), ("west", "false")]);
        assert!(is_bars(&pane.name));
        assert_eq!(connections(&pane), (1 << 2) | (1 << 5));
    }
}
