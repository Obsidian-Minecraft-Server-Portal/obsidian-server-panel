//! Blockstates choose which models a block draws, and models describe the boxes and faces of each model.

mod blockstate;
mod model;
mod resolve;

pub use blockstate::{Blockstate, Variant};
pub use model::{Element, ElementRotation, Face, ModelFile};
pub use resolve::{ResolvedModel, resolve_model};

/// Splits `namespace:path` and defaults the namespace to `minecraft`.
pub fn split_id(id: &str) -> (&str, &str) {
	id.split_once(':').unwrap_or(("minecraft", id))
}
