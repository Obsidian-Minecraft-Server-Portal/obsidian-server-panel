//! Bakes a block state into quads: its chosen variants or multipart cases, with their rotations applied, and the water it holds.

use std::collections::BTreeMap;

use crate::assets::AssetStore;
use crate::chunk::BlockState;
use crate::colour::block_tint;
use crate::models::geometry::{BlockTint, MISSING_TEXTURE, Quad, TintKind, element_quads, transform_quad};
use crate::models::{Blockstate, Element, Face, Variant, resolve_model, split_id};
use super::chest::chest_quads;
use super::cushion::cushion_quads;

pub fn bake_state(assets: &AssetStore, state: &BlockState) -> Vec<Quad> {
	if let Some((texture, tint)) = fluid_texture(&state.name) {
		return fluid_cube(texture, tint);
	}
	let mut quads = chest_quads(state)
		.or_else(|| cushion_quads(state))
		.unwrap_or_else(|| model_quads(assets, state, block_tint(&state.name, &state.properties)));
	if holds_water(state) {
		quads.extend(fluid_cube("minecraft:block/water_still", Some(BlockTint::Biome(TintKind::Water))));
	}
	quads
}

/// Blocks that hold water without being water: waterlogged blocks, and the plants and columns that grow in it.
pub fn holds_water(state: &BlockState) -> bool {
	state.properties.get("waterlogged").is_some_and(|value| value == "true")
		|| matches!(
			state.name.as_str(),
			"minecraft:seagrass" | "minecraft:tall_seagrass" | "minecraft:kelp" | "minecraft:kelp_plant" | "minecraft:bubble_column"
		)
}

fn model_quads(assets: &AssetStore, state: &BlockState, tint: Option<BlockTint>) -> Vec<Quad> {
	let (namespace, path) = split_id(&state.name);
	let variants = assets
		.get(namespace, &format!("blockstates/{path}.json"))
		.and_then(|json| Blockstate::parse(json).ok())
		.map(|blockstate| blockstate.variants_for(&state.properties));
	match variants {
		Some(variants) if !variants.is_empty() => variants.iter().flat_map(|variant| bake_variant(assets, variant, tint)).collect(),
		_ => cube(MISSING_TEXTURE, None),
	}
}

fn bake_variant(assets: &AssetStore, variant: &Variant, tint: Option<BlockTint>) -> Vec<Quad> {
	let Ok(model) = resolve_model(assets, &variant.model) else {
		return cube(MISSING_TEXTURE, None);
	};
	model
		.elements
		.iter()
		.flat_map(|element| element_quads(element, &model.textures, tint, model.ambient_occlusion))
		.map(|quad| transform_quad(&quad, variant.x, variant.y, variant.uvlock))
		.collect()
}

fn fluid_texture(name: &str) -> Option<(&'static str, Option<BlockTint>)> {
	match name {
		"minecraft:water" => Some(("minecraft:block/water_still", Some(BlockTint::Biome(TintKind::Water)))),
		"minecraft:lava" => Some(("minecraft:block/lava_still", None)),
		_ => None,
	}
}

/// A full cube of fluid. The browser lowers its top to the fluid's height, so the faces are marked as fluid.
fn fluid_cube(texture: &str, tint: Option<BlockTint>) -> Vec<Quad> {
	let mut quads = cube(texture, tint);
	for quad in &mut quads {
		quad.fluid = true;
	}
	quads
}

/// A full cube whose faces are all culled by their neighbours. Used for fluids and blocks with no usable model.
fn cube(texture: &str, tint: Option<BlockTint>) -> Vec<Quad> {
	let faces = ["down", "up", "north", "south", "west", "east"].map(|name| {
		let face = Face {
			uv: None,
			texture: "#all".to_owned(),
			cullface: Some(name.to_owned()),
			rotation: 0,
			tintindex: tint.map(|_| 0),
		};
		(name.to_owned(), face)
	});
	let element = Element {
		from: [0.0; 3],
		to: [16.0; 3],
		rotation: None,
		faces: BTreeMap::from(faces),
		shade: None,
		shade_direction_override: None,
	};
	let textures = BTreeMap::from([("all".to_owned(), texture.to_owned())]);
	element_quads(&element, &textures, tint, true)
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use std::io::Write;

	use super::*;
	use crate::models::geometry::Direction;

	fn store(files: &[(&str, &str)]) -> AssetStore {
		let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
		for (path, json) in files {
			writer.start_file(format!("assets/minecraft/{path}"), zip::write::SimpleFileOptions::default()).unwrap();
			writer.write_all(json.as_bytes()).unwrap();
		}
		let bytes = writer.finish().unwrap().into_inner();
		let dir = tempfile::tempdir().unwrap();
		let jar = dir.path().join("client.jar");
		std::fs::write(&jar, bytes).unwrap();
		let mut assets = AssetStore::default();
		assets.add_jar(&jar).unwrap();
		assets
	}

	fn state(name: &str, properties: &[(&str, &str)]) -> BlockState {
		BlockState {
			name: name.to_owned(),
			properties: properties.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect::<BTreeMap<_, _>>(),
		}
	}

	const MODEL: &str = r##"{"textures": {"front": "minecraft:block/front", "side": "minecraft:block/side"}, "elements": [{"from": [0,0,0], "to": [16,16,16], "faces": {"north": {"texture": "#front", "cullface": "north"}, "south": {"texture": "#side"}, "east": {"texture": "#side"}, "west": {"texture": "#side"}, "up": {"texture": "#side"}, "down": {"texture": "#side"}}}]}"##;

	#[test]
	fn applies_the_variant_rotation_to_the_model() {
		let assets = store(&[
			("blockstates/test_block.json", r#"{"variants": {"facing=north": {"model": "minecraft:block/test_model", "y": 90}, "": {"model": "minecraft:block/test_model"}}}"#),
			("models/block/test_model.json", MODEL),
		]);
		let quads = bake_state(&assets, &state("minecraft:test_block", &[("facing", "north")]));
		let front = quads.iter().find(|quad| quad.texture == "minecraft:block/front").unwrap();
		assert_eq!(front.direction, Direction::East);
		assert_eq!(quads.len(), 6);
	}

	#[test]
	fn falls_back_to_a_culled_cube_for_fluids_and_unknown_blocks() {
		let assets = AssetStore::default();
		let water = bake_state(&assets, &state("minecraft:water", &[("level", "0")]));
		assert_eq!(water.len(), 6);
		assert!(water.iter().all(|quad| quad.tint == Some(BlockTint::Biome(TintKind::Water)) && quad.texture == "minecraft:block/water_still"));
		assert!(water.iter().all(|quad| quad.fluid));
		let unknown = bake_state(&assets, &state("othermod:mystery", &[]));
		assert_eq!(unknown.len(), 6);
		assert!(unknown.iter().all(|quad| quad.texture == MISSING_TEXTURE && quad.cullface.is_some()));
	}

	#[test]
	fn waterlogged_blocks_hold_a_water_cube_beside_their_model() {
		let assets = store(&[
			("blockstates/test_block.json", r#"{"variants": {"": {"model": "minecraft:block/test_model"}}}"#),
			("models/block/test_model.json", MODEL),
		]);
		let quads = bake_state(&assets, &state("minecraft:test_block", &[("waterlogged", "true")]));
		assert_eq!(quads.iter().filter(|quad| quad.fluid).count(), 6);
		assert_eq!(quads.iter().filter(|quad| !quad.fluid).count(), 6);
		let dry = bake_state(&assets, &state("minecraft:test_block", &[("waterlogged", "false")]));
		assert!(dry.iter().all(|quad| !quad.fluid));
	}
}
