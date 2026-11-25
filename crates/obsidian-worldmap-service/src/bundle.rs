//! The structure of one cell, packed into a single response, so a viewer fetches a cell with one request. The bundle is built
//! from the section records a render has already written, so nothing is drawn here.

use std::path::{Path, PathBuf};

use obsidian_common::Result;
use obsidian_worldmap::structure::{SectionStore, assemble, section_path};

use crate::job::other;

/// The structure bundle of one cell: its sections and the border around them. Reads only the section files it needs.
pub fn structure_cell(folder_dir: &Path, cell: (i32, i32, i32)) -> Result<Vec<u8>> {
	let mut store = SectionFiles {
		folder: folder_dir.to_path_buf(),
	};
	assemble([cell.0, cell.1, cell.2], &mut store).map_err(other)
}

struct SectionFiles {
	folder: PathBuf,
}

impl SectionStore for SectionFiles {
	fn section(&mut self, chunk_x: i32, section_y: i32, chunk_z: i32) -> obsidian_worldmap::error::Result<Option<Vec<u8>>> {
		match std::fs::read(self.folder.join(section_path(chunk_x, section_y, chunk_z))) {
			Ok(bytes) => Ok(Some(bytes)),
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
			Err(error) => Err(error.into()),
		}
	}
}
