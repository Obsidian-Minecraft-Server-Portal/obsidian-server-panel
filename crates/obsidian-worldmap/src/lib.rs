//! Reads Minecraft region files and renders them into map tiles.

pub mod assets;
pub mod atlas;
pub mod biome;
pub mod chunk;
pub mod colour;
pub mod error;
mod jar;
pub mod level;
pub mod models;
pub mod players;
pub mod region;
pub mod render;
pub mod skins;
pub mod structure;
pub mod tiles;
pub mod world;

#[cfg(test)]
mod test_support;

pub use error::{Result, WorldmapError};
