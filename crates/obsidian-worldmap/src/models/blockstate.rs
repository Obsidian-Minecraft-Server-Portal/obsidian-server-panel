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

	fn all(&self) -> std::slice::Iter<'_, Variant> {
		match self {
			Self::Single(variant) => std::slice::from_ref(variant).iter(),
			Self::Many(list) => list.iter(),
		}
	}

	fn is_rotated(&self) -> bool {
		let variant = match self {
			Self::Single(variant) => Some(variant),
			Self::Many(list) => list.first(),
		};
		variant.is_some_and(|variant| variant.x != 0 || variant.y != 0)
	}
}

impl Blockstate {
	/// Every model the blockstate can draw, whatever the properties.
	pub fn model_ids(&self) -> Vec<&str> {
		self.variants
			.values()
			.flat_map(Choice::all)
			.chain(self.multipart.iter().flat_map(|part| part.apply.all()))
			.map(|variant| variant.model.as_str())
			.collect()
	}

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

	/// The properties a block takes when the chunk writes it as a bare name, its default state. The vanilla report lists
	/// those for vanilla blocks only, so a mod block gets a guess: the variant drawn unrotated, with booleans off and the
	/// usual value of each property. `None` when the bare name already matches and nothing needs filling in.
	pub fn default_properties(&self) -> Option<BTreeMap<String, String>> {
		if self.multipart.is_empty() {
			self.default_variant()
		} else {
			self.default_part()
		}
	}

	fn default_variant(&self) -> Option<BTreeMap<String, String>> {
		if self.variants.contains_key("") {
			return None;
		}
		let (key, _) = self
			.variants
			.iter()
			.min_by_key(|&(key, choice)| (choice.is_rotated(), unusual_values(key), key))?;
		Some(key_properties(key))
	}

	/// A multipart block with no properties can draw nothing, as a wall whose parts are all conditioned does. The first
	/// flag a part is conditioned on, set to true, is then the default that draws something.
	fn default_part(&self) -> Option<BTreeMap<String, String>> {
		if !self.variants_for(&BTreeMap::new()).is_empty() {
			return None;
		}
		self.multipart
			.iter()
			.filter_map(|part| part.when.as_ref()?.as_object())
			.flat_map(|conditions| conditions.iter().filter(|(_, value)| value.as_str() == Some("true")).map(|(name, _)| name.clone()))
			.map(|name| BTreeMap::from([(name, "true".to_owned())]))
			.find(|candidate| !self.variants_for(candidate).is_empty())
	}
}

/// Variant keys look like `facing=east,half=bottom`; the empty key matches every block.
fn variant_key_matches(key: &str, properties: &BTreeMap<String, String>) -> bool {
	key.split(',')
		.filter(|pair| !pair.is_empty())
		.all(|pair| pair.split_once('=').is_some_and(|(name, value)| properties.get(name).map(String::as_str) == Some(value)))
}

fn key_pairs(key: &str) -> impl Iterator<Item = (&str, &str)> + '_ {
	key.split(',').filter_map(|pair| pair.split_once('='))
}

fn key_properties(key: &str) -> BTreeMap<String, String> {
	key_pairs(key).map(|(name, value)| (name.to_owned(), value.to_owned())).collect()
}

fn unusual_values(key: &str) -> usize {
	key_pairs(key).filter(|&(name, value)| !is_usual(name, value)).count()
}

/// The values a mod author usually leaves a default state with: booleans off, and each property's first value as vanilla uses it.
fn is_usual(name: &str, value: &str) -> bool {
	matches!(
		(name, value),
		(_, "false") | ("axis", "y") | ("face", "wall") | ("facing", "north") | ("half", "bottom" | "lower") | ("hinge", "left") | ("shape", "straight") | ("type", "bottom" | "single")
	)
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

	#[test]
	fn a_bare_block_defaults_to_its_unrotated_off_variant() {
		let json = br#"{"variants": {
			"active=false,facing=east": {"model": "mod:block/smelter", "y": 90},
			"active=false,facing=north": {"model": "mod:block/smelter"},
			"active=false,facing=south": {"model": "mod:block/smelter", "y": 180},
			"active=false,facing=west": {"model": "mod:block/smelter", "y": 270},
			"active=true,facing=north": {"model": "mod:block/smelter_on"}
		}}"#;
		let defaults = Blockstate::parse(json).unwrap().default_properties();
		assert_eq!(defaults, Some(props(&[("active", "false"), ("facing", "north")])));
	}

	#[test]
	fn among_unrotated_variants_the_usual_values_win() {
		let stairs = br#"{"variants": {
			"facing=north,half=bottom,shape=inner_left": {"model": "mod:block/stairs_inner"},
			"facing=north,half=bottom,shape=straight": {"model": "mod:block/stairs"}
		}}"#;
		let button = br#"{"variants": {
			"face=ceiling,facing=north,powered=false": {"model": "mod:block/button_ceiling"},
			"face=wall,facing=north,powered=false": {"model": "mod:block/button"}
		}}"#;
		let stairs_default = Blockstate::parse(stairs).unwrap().default_properties();
		assert_eq!(stairs_default, Some(props(&[("facing", "north"), ("half", "bottom"), ("shape", "straight")])));
		let button_default = Blockstate::parse(button).unwrap().default_properties();
		assert_eq!(button_default, Some(props(&[("face", "wall"), ("facing", "north"), ("powered", "false")])));
	}

	#[test]
	fn a_blockstate_with_a_bare_variant_needs_no_default() {
		let json = br#"{"variants": {"": {"model": "mod:block/ore"}}}"#;
		assert_eq!(Blockstate::parse(json).unwrap().default_properties(), None);
	}

	#[test]
	fn a_bare_multipart_block_takes_the_flag_that_brings_a_part_in() {
		let wall = br#"{"multipart": [
			{"when": {"up": "true"}, "apply": {"model": "mod:block/post"}},
			{"when": {"north": "low"}, "apply": {"model": "mod:block/low"}}
		]}"#;
		let cable = br#"{"multipart": [
			{"apply": {"model": "mod:block/core"}},
			{"when": {"north": "true"}, "apply": {"model": "mod:block/arm"}}
		]}"#;
		assert_eq!(Blockstate::parse(wall).unwrap().default_properties(), Some(props(&[("up", "true")])));
		assert_eq!(Blockstate::parse(cable).unwrap().default_properties(), None);
	}
}
