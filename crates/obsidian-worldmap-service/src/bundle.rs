//! The structure of one cell, packed into a single response, so a viewer fetches a cell with one request. The bundle is built
//! from the section records a render has already written, so nothing is drawn here.

use std::path::Path;

use obsidian_common::Result;
use obsidian_worldmap::structure::cell_bundle;

use crate::job::other;

/// The structure bundle of one cell: its sections and the border around them.
pub fn structure_cell(folder_dir: &Path, cell: (i32, i32, i32)) -> Result<Vec<u8>> {
	cell_bundle(folder_dir, [cell.0, cell.1, cell.2]).map_err(other)
}
