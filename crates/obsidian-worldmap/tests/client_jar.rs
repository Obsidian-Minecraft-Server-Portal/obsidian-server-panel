//! Runs against a real Mojang client jar. Point OBSIDIAN_WORLDMAP_CLIENT_JAR at one and run with `--ignored`.

use std::collections::BTreeMap;
use std::path::Path;

use obsidian_worldmap::assets::AssetStore;
use obsidian_worldmap::biome::{Biome, BiomeRegistry};
use obsidian_worldmap::chunk::BlockState;
use obsidian_worldmap::colour::BlockColours;
use obsidian_worldmap::models::geometry::Direction;
use obsidian_worldmap::models::{Blockstate, bake_state, resolve_model};

fn client_jar_store() -> AssetStore {
	let jar = std::env::var("OBSIDIAN_WORLDMAP_CLIENT_JAR").expect("set OBSIDIAN_WORLDMAP_CLIENT_JAR");
	let mut store = AssetStore::default();
	store.add_jar(Path::new(&jar)).unwrap();
	store
}

fn block(name: &str, properties: &[(&str, &str)]) -> BlockState {
	BlockState {
		name: name.to_owned(),
		properties: properties.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect(),
	}
}

#[test]
#[ignore = "needs OBSIDIAN_WORLDMAP_CLIENT_JAR"]
fn resolves_oak_stairs_from_a_real_client_jar() {
	let store = client_jar_store();
	let json = store.get("minecraft", "blockstates/oak_stairs.json").expect("oak_stairs blockstate");
	let blockstate = Blockstate::parse(json).unwrap();
	let properties = BTreeMap::from([
		("facing".to_owned(), "east".to_owned()),
		("half".to_owned(), "bottom".to_owned()),
		("shape".to_owned(), "straight".to_owned()),
	]);
	let variant = blockstate.variants_for(&properties).into_iter().next().expect("a variant matches");
	let resolved = resolve_model(&store, &variant.model).unwrap();
	println!("{} -> {} elements, textures {:?}", variant.model, resolved.elements.len(), resolved.textures);
	assert!(!resolved.elements.is_empty());
	assert!(resolved.textures.values().any(|texture| texture.starts_with("minecraft:block/")));
}

#[test]
#[ignore = "needs OBSIDIAN_WORLDMAP_CLIENT_JAR"]
fn baked_rotations_match_vanilla_for_furnace_y_and_dropper_x() {
	let store = client_jar_store();
	let furnace = bake_state(&store, &block("minecraft:furnace", &[("facing", "east"), ("lit", "false")]));
	let front = furnace.iter().find(|quad| quad.texture == "minecraft:block/furnace_front").expect("furnace front face");
	assert_eq!(front.direction, Direction::East, "furnace facing=east is blockstate y=90");
	let observer = bake_state(&store, &block("minecraft:observer", &[("facing", "up"), ("powered", "false")]));
	let front = observer.iter().find(|quad| quad.texture == "minecraft:block/observer_front").expect("observer front face");
	assert_eq!(front.direction, Direction::Up, "observer facing=up is blockstate x=270");
	let dropper = bake_state(&store, &block("minecraft:dropper", &[("facing", "down"), ("triggered", "false")]));
	let front = dropper.iter().find(|quad| quad.texture == "minecraft:block/dropper_front_vertical").expect("dropper front face");
	assert_eq!(front.direction, Direction::Down, "dropper facing=down turns the front face to the bottom");
}

#[test]
#[ignore = "needs OBSIDIAN_WORLDMAP_CLIENT_JAR"]
fn colours_real_blocks_from_the_client_jar() {
	let store = client_jar_store();
	let biomes = BiomeRegistry::default();
	let mut colours = BlockColours::new(&store, &biomes);
	let plains = Biome::PLAINS;
	let samples = [
		block("minecraft:stone", &[]),
		block("minecraft:grass_block", &[("snowy", "false")]),
		block("minecraft:oak_leaves", &[("distance", "1"), ("persistent", "true")]),
		block("minecraft:water", &[("level", "0")]),
		block("minecraft:sand", &[]),
	];
	for sample in &samples {
		println!("{}: {:?}", sample.name, colours.colour(sample, &plains, 0, 0));
	}
	let [r, g, b] = colours.colour(&samples[1], &plains, 0, 0);
	assert!(g > r && g > b, "grass should be green, got {r} {g} {b}");
	let [r, _, b] = colours.colour(&samples[3], &plains, 0, 0);
	assert!(b > r, "water should be blue");
}
