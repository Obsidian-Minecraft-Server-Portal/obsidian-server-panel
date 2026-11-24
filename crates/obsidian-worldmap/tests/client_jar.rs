//! Runs against a real Mojang client jar. Point OBSIDIAN_WORLDMAP_CLIENT_JAR at one and run with `--ignored`.

use std::collections::BTreeMap;
use std::path::Path;

use obsidian_worldmap::assets::AssetStore;
use obsidian_worldmap::models::{Blockstate, resolve_model};

#[test]
#[ignore = "needs OBSIDIAN_WORLDMAP_CLIENT_JAR"]
fn resolves_oak_stairs_from_a_real_client_jar() {
	let jar = std::env::var("OBSIDIAN_WORLDMAP_CLIENT_JAR").expect("set OBSIDIAN_WORLDMAP_CLIENT_JAR");
	let mut store = AssetStore::default();
	store.add_jar(Path::new(&jar)).unwrap();
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
