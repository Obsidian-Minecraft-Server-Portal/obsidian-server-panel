//! Biome climate and colour overrides, the grass modifiers, and the registry that resolves biome ids.

mod definition;
mod registry;
mod swamp;

pub use definition::{Biome, GrassModifier};
pub use registry::BiomeRegistry;
