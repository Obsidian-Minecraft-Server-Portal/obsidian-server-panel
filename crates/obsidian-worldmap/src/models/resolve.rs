use std::collections::{BTreeMap, BTreeSet};

use crate::assets::AssetStore;
use crate::error::{Result, WorldmapError};
use crate::models::{Element, ModelFile, namespaced, split_id};

const MAX_PARENT_DEPTH: usize = 32;
const MAX_TEXTURE_HOPS: usize = 8;

/// A model with its parent chain merged in and every `#texture` variable resolved.
#[derive(Debug, Clone, Default)]
pub struct ResolvedModel {
	pub elements: Vec<Element>,
	pub textures: BTreeMap<String, String>,
	/// Sprites that a texture entry forces onto the translucent layer.
	pub translucent: BTreeSet<String>,
	pub ambient_occlusion: bool,
}

pub fn resolve_model(store: &AssetStore, model_id: &str) -> Result<ResolvedModel> {
	let mut chain: Vec<(String, ModelFile)> = Vec::new();
	let mut next = Some(model_id.to_owned());
	while let Some(id) = next.take() {
		if chain.len() == MAX_PARENT_DEPTH || chain.iter().any(|(seen, _)| *seen == id) {
			return Err(WorldmapError::Asset(format!("model {model_id} has a parent cycle")));
		}
		let file = load_model(store, &id)?;
		next = file.parent.clone().filter(|parent| !parent.starts_with("builtin/"));
		chain.push((id, file));
	}

	let mut textures = BTreeMap::new();
	let mut translucent = BTreeSet::new();
	let mut elements = Vec::new();
	for (_, file) in chain.iter().rev() {
		for (key, value) in &file.textures {
			textures.insert(key.clone(), value.id().to_owned());
			if value.force_translucent() {
				translucent.insert(namespaced(value.id()));
			}
		}
		if let Some(own) = &file.elements {
			elements = own.clone();
		}
	}
	Ok(ResolvedModel {
		elements,
		textures: resolve_texture_variables(textures),
		translucent,
		ambient_occlusion: chain.iter().find_map(|(_, file)| file.ambientocclusion).unwrap_or(true),
	})
}

fn load_model(store: &AssetStore, id: &str) -> Result<ModelFile> {
	let (namespace, path) = split_id(id);
	let bytes = store
		.get(namespace, &format!("models/{path}.json"))
		.ok_or_else(|| WorldmapError::Asset(format!("model {id} is not in the asset store")))?;
	serde_json::from_slice(bytes).map_err(|error| WorldmapError::Asset(format!("model {id}: {error}")))
}

/// Replaces `#name` references with their targets, following chains up to a fixed depth.
fn resolve_texture_variables(mut textures: BTreeMap<String, String>) -> BTreeMap<String, String> {
	for _ in 0..MAX_TEXTURE_HOPS {
		let snapshot = textures.clone();
		for value in textures.values_mut() {
			if let Some(target) = value.strip_prefix('#').and_then(|key| snapshot.get(key)) {
				*value = target.clone();
			}
		}
	}
	textures
}

#[cfg(test)]
mod tests {
	use super::*;

	fn store_with(models: &[(&str, &str)]) -> AssetStore {
		let dir = tempfile::tempdir().unwrap();
		let jar = dir.path().join("client.jar");
		let mut writer = zip::ZipWriter::new(std::fs::File::create(&jar).unwrap());
		for (path, json) in models {
			use std::io::Write;
			writer.start_file(format!("assets/minecraft/models/{path}.json"), zip::write::SimpleFileOptions::default()).unwrap();
			writer.write_all(json.as_bytes()).unwrap();
		}
		writer.finish().unwrap();
		let mut store = AssetStore::default();
		store.add_jar(&jar).unwrap();
		store
	}

	#[test]
	fn inherits_elements_and_textures_from_parents_and_resolves_variables() {
		let store = store_with(&[
			(
				"block/cube",
				r##"{"textures": {"particle": "#all"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#up"}}}]}"##,
			),
			("block/cube_all", r##"{"parent": "block/cube", "textures": {"all": "minecraft:block/stone", "up": "#all"}}"##),
			("block/oak_log", r##"{"parent": "minecraft:block/cube_all", "textures": {"all": "minecraft:block/oak_log_top"}}"##),
		]);
		let resolved = resolve_model(&store, "minecraft:block/oak_log").unwrap();
		assert_eq!(resolved.elements.len(), 1);
		assert_eq!(resolved.textures["up"], "minecraft:block/oak_log_top");
		assert_eq!(resolved.textures["particle"], "minecraft:block/oak_log_top");
	}

	#[test]
	fn texture_objects_keep_their_sprite_and_mark_it_translucent() {
		let store = store_with(&[
			(
				"block/cube",
				r##"{"textures": {"particle": "#all"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"up": {"texture": "#all"}}}]}"##,
			),
			(
				"block/glass",
				r#"{"parent": "minecraft:block/cube", "textures": {"all": {"sprite": "minecraft:block/glass", "force_translucent": true}}}"#,
			),
		]);
		let resolved = resolve_model(&store, "minecraft:block/glass").unwrap();
		assert_eq!(resolved.textures["all"], "minecraft:block/glass");
		assert!(resolved.translucent.contains("minecraft:block/glass"));
	}

	#[test]
	fn ambient_occlusion_is_inherited_and_defaults_to_on() {
		let store = store_with(&[
			("block/flat", r#"{"ambientocclusion": false, "elements": []}"#),
			("block/plant", r#"{"parent": "minecraft:block/flat"}"#),
			("block/cube", r#"{"elements": []}"#),
		]);
		assert!(!resolve_model(&store, "minecraft:block/plant").unwrap().ambient_occlusion);
		assert!(resolve_model(&store, "minecraft:block/cube").unwrap().ambient_occlusion);
	}

	#[test]
	fn reports_parent_cycles_instead_of_looping() {
		let store = store_with(&[("block/a", r#"{"parent": "minecraft:block/b"}"#), ("block/b", r#"{"parent": "minecraft:block/a"}"#)]);
		assert!(resolve_model(&store, "minecraft:block/a").is_err());
	}
}
