//! Blockstates choose which models a block draws, and models describe the boxes and faces of each model.

mod bake;
mod blockstate;
mod chest;
mod cushion;
mod entity;
pub mod geometry;
mod model;
mod resolve;

pub use bake::{bake_state, holds_water};
pub use blockstate::{Blockstate, Variant};
pub use cushion::STATE as CUSHION_STATE;
pub use model::{Element, ElementRotation, Face, ModelFile, RotationValue};
pub use resolve::{ResolvedModel, resolve_model};

/// Splits `namespace:path` and defaults the namespace to `minecraft`.
pub fn split_id(id: &str) -> (&str, &str) {
	id.split_once(':').unwrap_or(("minecraft", id))
}

/// Gives an id without a namespace the `minecraft` namespace, as the game does.
pub fn namespaced(id: &str) -> String {
	if id.contains(':') {
		id.to_owned()
	} else {
		format!("minecraft:{id}")
	}
}

/// The textures of the entities the structure draws. The atlas packs each one at its full 64 texel size.
pub fn entity_textures() -> Vec<String> {
	chest::TEXTURES.iter().map(|texture| (*texture).to_owned()).chain(cushion::textures()).collect()
}
