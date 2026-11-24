//! Finds every dimension that stores region files, across the folder layouts vanilla, modded and
//! Bukkit-style worlds use.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::Result;

const OVERWORLD: &str = "minecraft:overworld";
const NETHER: &str = "minecraft:the_nether";
const END: &str = "minecraft:the_end";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dimension {
	pub id: String,
	pub region_dir: PathBuf,
}

/// Overworld first, then Nether and End, then every other dimension by id.
pub fn discover(server_dir: &Path, level_name: &str) -> Result<Vec<Dimension>> {
	let world = server_dir.join(level_name);
	let mut found = Vec::new();
	push_if_present(&mut found, OVERWORLD, world.join("region"));
	push_if_present(&mut found, NETHER, world.join("DIM-1").join("region"));
	push_if_present(&mut found, END, world.join("DIM1").join("region"));
	push_if_present(&mut found, NETHER, server_dir.join(format!("{level_name}_nether")).join("region"));
	push_if_present(&mut found, END, server_dir.join(format!("{level_name}_the_end")).join("region"));
	scan_dimensions_tree(&world.join("dimensions"), &mut found)?;
	found.sort_by(|a, b| rank(&a.id).cmp(&rank(&b.id)).then_with(|| a.id.cmp(&b.id)));
	Ok(found)
}

fn push_if_present(found: &mut Vec<Dimension>, id: &str, region_dir: PathBuf) {
	if region_dir.is_dir() && !found.iter().any(|dimension| dimension.id == id) {
		found.push(Dimension {
			id: id.to_owned(),
			region_dir,
		});
	}
}

/// 1.16 and later store dimensions at `dimensions/<namespace>/<path>/region`.
fn scan_dimensions_tree(root: &Path, found: &mut Vec<Dimension>) -> Result<()> {
	if !root.is_dir() {
		return Ok(());
	}
	for entry in WalkDir::new(root).min_depth(1).max_depth(6).sort_by_file_name() {
		let entry = entry.map_err(std::io::Error::from)?;
		if !entry.file_type().is_dir() || entry.file_name() != OsStr::new("region") {
			continue;
		}
		let Some(dimension_dir) = entry.path().parent() else { continue };
		let Ok(relative) = dimension_dir.strip_prefix(root) else { continue };
		let mut parts = relative.components().map(|part| part.as_os_str().to_string_lossy().into_owned());
		let (Some(namespace), Some(first_path)) = (parts.next(), parts.next()) else { continue };
		let id = std::iter::once(first_path).chain(parts).collect::<Vec<_>>().join("/");
		push_if_present(found, &format!("{namespace}:{id}"), entry.path().to_owned());
	}
	Ok(())
}

fn rank(id: &str) -> u8 {
	match id {
		OVERWORLD => 0,
		NETHER => 1,
		END => 2,
		_ => 3,
	}
}

#[cfg(test)]
mod tests {
	use std::fs;

	use super::*;

	fn make_dirs(root: &Path, relative: &[&str]) {
		for path in relative {
			fs::create_dir_all(root.join(path)).unwrap();
		}
	}

	fn ids(dimensions: &[Dimension]) -> Vec<&str> {
		dimensions.iter().map(|dimension| dimension.id.as_str()).collect()
	}

	#[test]
	fn finds_legacy_folders_and_orders_the_vanilla_dimensions_first() {
		let dir = tempfile::tempdir().unwrap();
		make_dirs(dir.path(), &["world/DIM1/region", "world/DIM-1/region", "world/region"]);
		let found = discover(dir.path(), "world").unwrap();
		assert_eq!(ids(&found), ["minecraft:overworld", "minecraft:the_nether", "minecraft:the_end"]);
		assert_eq!(found[1].region_dir, dir.path().join("world/DIM-1/region"));
	}

	#[test]
	fn finds_26x_folders_and_modded_dimensions() {
		let dir = tempfile::tempdir().unwrap();
		make_dirs(
			dir.path(),
			&[
				"world/dimensions/minecraft/overworld/region",
				"world/dimensions/minecraft/the_nether/region",
				"world/dimensions/mymod/cave/deep/region",
				"world/dimensions/mymod/empty/data",
			],
		);
		let found = discover(dir.path(), "world").unwrap();
		assert_eq!(ids(&found), ["minecraft:overworld", "minecraft:the_nether", "mymod:cave/deep"]);
	}

	#[test]
	fn reads_bukkit_style_sibling_folders() {
		let dir = tempfile::tempdir().unwrap();
		make_dirs(dir.path(), &["world/region", "world_nether/region", "world_the_end/region"]);
		let found = discover(dir.path(), "world").unwrap();
		assert_eq!(ids(&found), ["minecraft:overworld", "minecraft:the_nether", "minecraft:the_end"]);
		assert_eq!(found[1].region_dir, dir.path().join("world_nether/region"));
	}
}
