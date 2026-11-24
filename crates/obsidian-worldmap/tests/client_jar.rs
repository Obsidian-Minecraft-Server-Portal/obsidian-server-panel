//! Runs against a real Mojang client jar. Point OBSIDIAN_WORLDMAP_CLIENT_JAR at one and run with `--ignored`.

use std::collections::BTreeMap;
use std::path::Path;

use obsidian_worldmap::assets::AssetStore;
use obsidian_worldmap::chunk::BlockState;
use obsidian_worldmap::colour::BlockColours;
use obsidian_worldmap::models::{Blockstate, resolve_model};

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
fn colours_real_blocks_from_the_client_jar() {
	let store = client_jar_store();
	let mut colours = BlockColours::new(&store);
	let samples = [
		block("minecraft:stone", &[]),
		block("minecraft:grass_block", &[("snowy", "false")]),
		block("minecraft:oak_leaves", &[("distance", "1"), ("persistent", "true")]),
		block("minecraft:water", &[("level", "0")]),
		block("minecraft:sand", &[]),
	];
	for sample in &samples {
		println!("{}: {:?}", sample.name, colours.colour(sample));
	}
	let [r, g, b] = colours.colour(&samples[1]);
	assert!(g > r && g > b, "grass should be green, got {r} {g} {b}");
	let [r, _, b] = colours.colour(&samples[3]);
	assert!(b > r, "water should be blue");
}
