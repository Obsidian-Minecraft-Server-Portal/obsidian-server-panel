//! Writes the texture atlas, one binary mesh per non-empty section, and a manifest for one region.
//! The viewer harness reads this output.
//!
//! Usage: cargo run -p obsidian-worldmap --example export_region_mesh -- <output dir> <blocks.json> <region.mca> <client.jar> [mod jars...]
//! Mod jars load after the client jar, so they override its assets the way the game does.
//! The blocks report comes from the vanilla data generator: `java -DbundlerMainClass=net.minecraft.data.Main
//! -jar server.jar --reports --output <dir>`, which writes `<dir>/reports/blocks.json`.

use std::path::PathBuf;

use obsidian_worldmap::assets::AssetStore;
use obsidian_worldmap::chunk::{BlockDefaults, Chunk};
use obsidian_worldmap::mesh::{BlockWorld, MeshContext, format};
use obsidian_worldmap::region::{CHUNKS_PER_SIDE, RegionFile};

fn main() -> obsidian_worldmap::Result<()> {
	let args: Vec<String> = std::env::args().skip(1).collect();
	if args.len() < 4 {
		eprintln!("usage: export_region_mesh <output dir> <blocks.json> <region.mca> <client.jar> [mod jars...]");
		std::process::exit(2);
	}
	let output = PathBuf::from(&args[0]);
	std::fs::create_dir_all(&output)?;
	let defaults = BlockDefaults::parse(&std::fs::read(&args[1])?)?;

	let mut assets = AssetStore::default();
	assets.add_jar(&PathBuf::from(&args[3]))?;
	for jar in &args[4..] {
		assets.add_jar(&PathBuf::from(jar))?;
	}
	let region = RegionFile::open(&PathBuf::from(&args[2]))?;
	let mut chunks = Vec::new();
	for local_z in 0..CHUNKS_PER_SIDE {
		for local_x in 0..CHUNKS_PER_SIDE {
			if let Some(nbt) = region.chunk_nbt(local_x, local_z)? {
				chunks.push(Chunk::parse(&nbt)?);
			}
		}
	}
	let world = BlockWorld::new(chunks, &defaults);
	let sections: Vec<(i32, i32, i32)> = world
		.chunks()
		.flat_map(|chunk| chunk.sections().iter().map(move |section| (chunk.x, section.y(), chunk.z)))
		.collect();

	let mut context = MeshContext::new(&assets, &world);
	context.atlas().image().save(output.join("atlas.png"))?;
	let (atlas_width, atlas_height) = context.atlas().image().dimensions();
	let mut entries = Vec::new();
	for (x, y, z) in sections {
		let Some(mesh) = context.mesh_section(&world, x, y, z) else { continue };
		let quads: usize = mesh.layers.iter().map(Vec::len).sum();
		if quads == 0 {
			continue;
		}
		let file = format!("section_{x}_{y}_{z}.obsm");
		std::fs::write(output.join(&file), format::encode(&mesh))?;
		entries.push(serde_json::json!({"x": x, "y": y, "z": z, "file": file, "quads": quads}));
	}
	let manifest = serde_json::json!({
		"atlas": "atlas.png",
		"atlasWidth": atlas_width,
		"atlasHeight": atlas_height,
		"sections": entries,
	});
	std::fs::write(output.join("manifest.json"), serde_json::to_vec_pretty(&manifest).map_err(std::io::Error::other)?)?;
	eprintln!("wrote {} sections and the atlas to {}", entries.len(), output.display());
	Ok(())
}
