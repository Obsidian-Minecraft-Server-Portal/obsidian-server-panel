//! Default property values per block, from the vanilla data generator's `blocks.json` report. Version 26.x writes a
//! block in its default state as a bare name, so those properties must be filled in before baking.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::chunk::BlockState;
use crate::error::{Result, WorldmapError};

#[derive(Debug, Clone, Default)]
pub struct BlockDefaults {
	properties: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Deserialize)]
struct ReportBlock {
	#[serde(default)]
	states: Vec<ReportState>,
}

#[derive(Deserialize)]
struct ReportState {
	#[serde(default)]
	default: bool,
	#[serde(default)]
	properties: BTreeMap<String, String>,
}

impl BlockDefaults {
	pub fn parse(blocks_json: &[u8]) -> Result<Self> {
		let blocks: BTreeMap<String, ReportBlock> = serde_json::from_slice(blocks_json)
			.map_err(|error| WorldmapError::Asset(format!("blocks report: {error}")))?;
		let properties = blocks
			.into_iter()
			.filter_map(|(name, block)| {
				let default = block.states.into_iter().find(|state| state.default)?;
				Some((name, default.properties))
			})
			.collect();
		Ok(Self { properties })
	}

	/// Adds the default value of every property the state leaves out. Properties it already has are kept.
	pub fn fill(&self, state: &mut BlockState) {
		if let Some(defaults) = self.properties.get(&state.name) {
			for (name, value) in defaults {
				state.properties.entry(name.clone()).or_insert_with(|| value.clone());
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fills_only_the_properties_a_bare_name_leaves_out() {
		let json = br#"{"minecraft:oak_log": {"states": [{"id": 1, "default": true, "properties": {"axis": "y"}}, {"id": 2, "properties": {"axis": "x"}}]}, "minecraft:stone": {"states": [{"id": 3, "default": true}]}}"#;
		let defaults = BlockDefaults::parse(json).unwrap();
		let mut bare = BlockState {
			name: "minecraft:oak_log".into(),
			properties: BTreeMap::new(),
		};
		defaults.fill(&mut bare);
		assert_eq!(bare.properties.get("axis").map(String::as_str), Some("y"));
		let mut explicit = BlockState {
			name: "minecraft:oak_log".into(),
			properties: BTreeMap::from([("axis".to_owned(), "x".to_owned())]),
		};
		defaults.fill(&mut explicit);
		assert_eq!(explicit.properties.get("axis").map(String::as_str), Some("x"));
	}
}
