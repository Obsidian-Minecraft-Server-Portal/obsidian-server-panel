//! Textured detail tiles. A region's tiles are drawn together the first time one of them is asked for, under the time of the
//! render they belong to, so a region that renders again is never answered with tiles of its earlier render.

use std::path::{Path, PathBuf};
use std::sync::PoisonError;

use obsidian_common::{CommonError, Result};
use obsidian_worldmap::atomic;
use obsidian_worldmap::region::RegionFile;
use obsidian_worldmap::tiles::{detail::TILES_PER_SIDE, encode_png};

use crate::job::{dimension_folder, other};
use crate::manifest;
use crate::pack_cache::PackHandle;

/// Where a tile is cached, after checking that the folder is a plain name. The numbers cannot escape the folder.
pub fn tile_path(map_dir: &Path, folder: &str, region: (i32, i32), tile: (usize, usize), version: i64) -> Result<PathBuf> {
    if tile.0 >= TILES_PER_SIDE || tile.1 >= TILES_PER_SIDE {
        return Err(CommonError::Invalid(format!("detail tile {tile:?} is outside its region")));
    }
    let folder_dir = manifest::file_path(map_dir, folder)?;
    Ok(folder_dir
        .join("detail")
        .join(format!("r.{}.{}", region.0, region.1))
        .join(version.to_string())
        .join(format!("{}_{}.png", tile.0, tile.1)))
}

/// The region file of `folder`'s dimension, which must exist.
pub fn region_path(instance_dir: &Path, level: &str, folder: &str, region: (i32, i32)) -> Result<PathBuf> {
    let dimensions = obsidian_worldmap::level::discover(instance_dir, level).map_err(other)?;
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

/// Draws every tile of one region with the pack its render used, and writes them. The surface cache is locked only while
/// one tile is drawn, so PNG encoding and file writes never hold it. Runs on a blocking thread.
pub fn draw_region(pack: &PackHandle, map_dir: &Path, folder: &str, region: &Path, coords: (i32, i32), version: i64) -> Result<()> {
    let region_file = RegionFile::open(region).map_err(other)?;
    for tile_z in 0..TILES_PER_SIDE {
        for tile_x in 0..TILES_PER_SIDE {
            let image = {
                let mut surfaces = pack.surfaces.lock().unwrap_or_else(PoisonError::into_inner);
                pack.pack
                    .render_detail_tile(&mut surfaces, &region_file, tile_x, tile_z)
                    .map_err(other)?
            };
            let location = tile_path(map_dir, folder, coords, (tile_x, tile_z), version)?;
            atomic::write(&location, encode_png(&image).map_err(other)?)?;
        }
    }
    Ok(())
}
