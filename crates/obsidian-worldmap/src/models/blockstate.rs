use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::{Result, WorldmapError};

/// `assets/<namespace>/blockstates/<block>.json`: either `variants` keyed by property strings, or `multipart`.
#[derive(Debug, Deserialize)]
pub struct Blockstate {
	#[serde(default)]
	variants: BTreeMap<String, Choice>,
	#[serde(default)]
	multipart: Vec<Part>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Variant {
	pub model: String,
	#[serde(default)]
	pub x: i32,
	#[serde(default)]
	pub y: i32,
	#[serde(default)]
	pub uvlock: bool,
	#[serde(default = "one")]
	pub weight: u32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Choice {
	Single(Variant),
	Many(Vec<Variant>),
}

#[derive(Debug, Deserialize)]
struct Part {
	#[serde(default)]
	when: Option<serde_json::Value>,
	apply: Choice,
}

fn one() -> u32 {
	1
}

impl Choice {
	/// Weighted random choice in the game; the first entry is used here so output is deterministic.
	fn first(&self) -> Option<Variant> {
		match self {
			Self::Single(variant) => Some(variant.clone()),
			Self::Many(list) => list.first().cloned(),
		}
	}
}

impl Blockstate {
	pub fn parse(json: &[u8]) -> Result<Self> {
		serde_json::from_slice(json).map_err(|error| WorldmapError::Asset(format!("blockstate JSON: {error}")))
	}

	/// The model parts a block with these properties draws.
	pub fn variants_for(&self, properties: &BTreeMap<String, String>) -> Vec<Variant> {
		if self.multipart.is_empty() {
			return self
				.variants
				.iter()
				.find(|(key, _)| !key.is_empty() && variant_key_matches(key, properties))
				.or_else(|| self.variants.iter().find(|(key, _)| key.is_empty()))
				.and_then(|(_, choice)| choice.first())
				.into_iter()
				.collect();
		}
		self.multipart
			.iter()
			.filter(|part| part.when.as_ref().is_none_or(|when| condition_matches(when, properties)))
			.filter_map(|part| part.apply.first())
			.collect()
	}
}

/// Variant keys look like `facing=east,half=bottom`; the empty key matches every block.
fn variant_key_matches(key: &str, properties: &BTreeMap<String, String>) -> bool {
	key.split(',')
		.filter(|pair| !pair.is_empty())
		.all(|pair| pair.split_once('=').is_some_and(|(name, value)| properties.get(name).map(String::as_str) == Some(value)))
}

/// Multipart conditions: `{"facing": "north|east"}`, plus `OR` and `AND` lists of conditions.
fn condition_matches(condition: &serde_json::Value, properties: &BTreeMap<String, String>) -> bool {
	let Some(map) = condition.as_object() else { return true };
	map.iter().all(|(name, expected)| match name.as_str() {
		"OR" => expected.as_array().is_some_and(|items| items.iter().any(|item| condition_matches(item, properties))),
		"AND" => expected.as_array().is_some_and(|items| items.iter().all(|item| condition_matches(item, properties))),
		property => {
			let actual = properties.get(property).map_or("", String::as_str);
			let wanted = expected.as_str().map_or_else(|| expected.to_string(), str::to_owned);
			wanted.split('|').any(|value| value == actual)
		}
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	fn props(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
		pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect()
	}

	#[test]
	fn selects_the_variant_whose_properties_match() {
		let json = br#"{"variants": {
			"facing=north,half=bottom": {"model": "minecraft:block/a"},
			"facing=east,half=bottom": {"model": "minecraft:block/b", "y": 90},
			"": {"model": "minecraft:block/c"}
		}}"#;
		let blockstate = Blockstate::parse(json).unwrap();
		let chosen = blockstate.variants_for(&props(&[("facing", "east"), ("half", "bottom")]));
		assert_eq!(chosen.len(), 1);
		assert_eq!((chosen[0].model.as_str(), chosen[0].y), ("minecraft:block/b", 90));
		let fallback = blockstate.variants_for(&props(&[("facing", "west")]));
		assert_eq!(fallback[0].model, "minecraft:block/c");
	}

	#[test]
	fn applies_every_multipart_case_whose_condition_holds() {
		let json = br#"{"multipart": [
			{"when": {"OR": [{"facing": "north"}, {"facing": "east"}]}, "apply": {"model": "minecraft:block/x"}},
			{"apply": {"model": "minecraft:block/y", "weight": 2}},
			{"when": {"facing": "west"}, "apply": {"model": "minecraft:block/z"}}
		]}"#;
		let blockstate = Blockstate::parse(json).unwrap();
		let models: Vec<String> = blockstate
			.variants_for(&props(&[("facing", "east")]))
			.into_iter()
			.map(|variant| variant.model)
			.collect();
		assert_eq!(models, ["minecraft:block/x", "minecraft:block/y"]);
	}
}
