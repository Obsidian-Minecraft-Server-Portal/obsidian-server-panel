//! Textured detail tiles. A tile is drawn the first time a viewer asks for it, from the pack its region was rendered with,
//! and kept beside the region's other files until the region renders again.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_worldmap::colour::SurfaceCache;
use obsidian_worldmap::level::discover;
use obsidian_worldmap::region::RegionFile;
use obsidian_worldmap::render::{Pack, PackSources};
use obsidian_worldmap::tiles::{detail::TILES_PER_SIDE, encode_png};

use crate::job::{Sources, dimension_folder, other};
use crate::manifest;

/// A pack that has not drawn a tile for this long is dropped, so its assets leave memory with the viewers.
const IDLE_LIMIT: Duration = Duration::from_secs(10 * 60);

pub struct LoadedPack {
	key: String,
	pack: Pack,
	cache: SurfaceCache,
	used: Instant,
}

pub type Packs = Mutex<HashMap<InstanceId, LoadedPack>>;

/// Where a tile is cached, after checking that the folder is a plain name. The numbers cannot escape the folder.
pub fn tile_path(map_dir: &Path, folder: &str, region: (i32, i32), tile: (usize, usize)) -> Result<PathBuf> {
	if tile.0 >= TILES_PER_SIDE || tile.1 >= TILES_PER_SIDE {
		return Err(CommonError::Invalid(format!("detail tile {tile:?} is outside its region")));
	}
	let folder_dir = manifest::file_path(map_dir, folder)?;
	Ok(folder_dir
		.join("detail")
		.join(format!("r.{}.{}", region.0, region.1))
		.join(format!("{}_{}.png", tile.0, tile.1)))
}

/// The region file of `folder`'s dimension, which must exist.
pub fn region_path(instance_dir: &Path, level: &str, folder: &str, region: (i32, i32)) -> Result<PathBuf> {
	let dimensions = discover(instance_dir, level).map_err(other)?;
	let dimension = dimensions
		.into_iter()
		.find(|dimension| dimension_folder(&dimension.id) == folder)
		.ok_or_else(|| CommonError::NotFound(format!("no dimension {folder}")))?;
	let path = dimension.region_dir.join(format!("r.{}.{}.mca", region.0, region.1));
	if !path.is_file() {
		return Err(CommonError::NotFound(format!("region r.{}.{} is not in {folder}", region.0, region.1)));
	}
	Ok(path)
}

/// Draws one tile and encodes it as PNG. Runs on a blocking thread: loading a pack reads the jars.
pub fn draw(packs: &Packs, id: InstanceId, key: &str, sources: &Sources, region: &Path, tile: (usize, usize)) -> Result<Vec<u8>> {
	let mut loaded = packs.lock().unwrap_or_else(PoisonError::into_inner);
	loaded.retain(|_, entry| entry.used.elapsed() < IDLE_LIMIT);
	if loaded.get(&id).is_none_or(|entry| entry.key != key) {
		let pack = Pack::load(&PackSources {
			client_jar: &sources.client_jar,
			server_jar: &sources.server_jar,
			mod_jars: &sources.mod_jars,
			blocks_report: &sources.blocks_report,
		})
		.map_err(other)?;
		loaded.insert(
			id,
			LoadedPack {
				key: key.to_owned(),
				pack,
				cache: SurfaceCache::default(),
				used: Instant::now(),
			},
		);
	}
	let entry = loaded.get_mut(&id).ok_or_else(|| CommonError::Other("the pack did not load".into()))?;
	entry.used = Instant::now();
	let region = RegionFile::open(region).map_err(other)?;
	let image = entry.pack.render_detail_tile(&mut entry.cache, &region, tile.0, tile.1).map_err(other)?;
	encode_png(&image).map_err(other)
}

pub fn release_idle(packs: &Packs) {
	packs.lock().unwrap_or_else(PoisonError::into_inner).retain(|_, entry| entry.used.elapsed() < IDLE_LIMIT);
}
