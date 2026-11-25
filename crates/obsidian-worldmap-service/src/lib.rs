//! Keeps each instance's world map rendered: the settings that control it, the consent-gated Mojang jars it needs,
//! and the render jobs that run when the world changes, when a server stops, and on a full pass.

mod job;
mod jars;
mod live;
mod manifest;
mod policy;
mod service;
mod settings;
mod state;
mod status;

#[cfg(test)]
mod tests;

pub use live::{LivePlayer, LiveState, WorldMapPlayers};
pub use manifest::{WorldMapManifest, WorldMapSection, WorldMapTile};
pub use service::WorldMapService;
pub use settings::WorldMapSettings;
pub use status::{WorldMapDimensionStatus, WorldMapStatus};
