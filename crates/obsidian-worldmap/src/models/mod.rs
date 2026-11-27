//! Blockstates choose which models a block draws, and models describe the boxes and faces of each model.

mod bake;
mod blockstate;
pub mod geometry;
mod model;
mod resolve;

pub use bake::{bake_state, holds_water};
pub use blockstate::{Blockstate, Variant};
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
