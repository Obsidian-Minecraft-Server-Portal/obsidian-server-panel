mod codec;
mod lz4;

use std::path::{Path, PathBuf};

use crate::error::{Result, WorldmapError};

pub use codec::Compression;

pub const CHUNKS_PER_SIDE: usize = 32;
const SECTOR_LEN: usize = 4096;
const HEADER_LEN: usize = 2 * SECTOR_LEN;
const EXTERNAL_FLAG: u8 = 0x80;

/// One `r.X.Z.mca` file. Chunk payloads are decompressed on demand.
pub struct RegionFile {
	path: PathBuf,
	bytes: Vec<u8>,
	region_x: i32,
	region_z: i32,
}

impl RegionFile {
	pub fn open(path: &Path) -> Result<Self> {
		let (region_x, region_z) = parse_coords(path)?;
		let bytes = std::fs::read(path)?;
		if bytes.len() < HEADER_LEN {
			return Err(WorldmapError::Region {
				path: path.to_owned(),
				reason: "shorter than its 8 KiB header".into(),
			});
		}
		Ok(Self {
			path: path.to_owned(),
			bytes,
			region_x,
			region_z,
		})
	}

	pub fn region_coords(&self) -> (i32, i32) {
		(self.region_x, self.region_z)
	}

	/// Decompressed NBT for one chunk, or `None` when the chunk has never been generated.
	pub fn chunk_nbt(&self, local_x: usize, local_z: usize) -> Result<Option<Vec<u8>>> {
		if local_x >= CHUNKS_PER_SIDE || local_z >= CHUNKS_PER_SIDE {
			return Err(WorldmapError::Chunk(format!("local chunk ({local_x}, {local_z}) is outside the region")));
		}
		let entry = self.read_u32((local_x + local_z * CHUNKS_PER_SIDE) * 4)?;
		let sector = (entry >> 8) as usize;
		if sector == 0 {
			return Ok(None);
		}
		let start = sector * SECTOR_LEN;
		let length = self.read_u32(start)? as usize;
		if length == 0 {
			return Ok(None);
		}
		let id = self.slice(start + 4, start + 5)?[0];
		let compression = Compression::from_id(id & !EXTERNAL_FLAG)?;
		if id & EXTERNAL_FLAG == 0 {
			return compression.decode(self.slice(start + 5, start + 4 + length)?).map(Some);
		}
		let external = self.external_path(local_x, local_z);
		let stored = std::fs::read(&external).map_err(|error| WorldmapError::Region {
			path: external.clone(),
			reason: format!("external chunk is missing: {error}"),
		})?;
		compression.decode(&stored).map(Some)
	}

	fn read_u32(&self, offset: usize) -> Result<u32> {
		let bytes = self.slice(offset, offset + 4)?;
		Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
	}

	fn slice(&self, start: usize, end: usize) -> Result<&[u8]> {
		self.bytes.get(start..end).ok_or_else(|| WorldmapError::Region {
			path: self.path.clone(),
			reason: format!("bytes {start}..{end} are past the end of the file"),
		})
	}

	fn external_path(&self, local_x: usize, local_z: usize) -> PathBuf {
		let x = self.region_x * CHUNKS_PER_SIDE as i32 + local_x as i32;
		let z = self.region_z * CHUNKS_PER_SIDE as i32 + local_z as i32;
		self.path.with_file_name(format!("c.{x}.{z}.mcc"))
	}
}

fn parse_coords(path: &Path) -> Result<(i32, i32)> {
	let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
	let parsed = name
		.strip_prefix("r.")
		.and_then(|rest| rest.strip_suffix(".mca"))
		.and_then(|coords| coords.split_once('.'))
		.and_then(|(x, z)| Some((x.parse::<i32>().ok()?, z.parse::<i32>().ok()?)));
	parsed.ok_or_else(|| WorldmapError::Region {
		path: path.to_owned(),
		reason: "file name is not r.<x>.<z>.mca".into(),
	})
}

#[cfg(test)]
mod tests {
	use std::io::Write;

	use super::*;
	use crate::chunk::Chunk;
	use crate::test_support::chunk_nbt;

	fn zlib(bytes: &[u8]) -> Vec<u8> {
		let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
		encoder.write_all(bytes).unwrap();
		encoder.finish().unwrap()
	}

	/// A header whose chunk (0, 0) sits in sector 2 as `length`, `id`, then `payload`.
	fn region_with_chunk(id: u8, payload: &[u8]) -> Vec<u8> {
		let mut file = vec![0_u8; HEADER_LEN];
		file[0..4].copy_from_slice(&((2_u32 << 8) | 1).to_be_bytes());
		let mut sector = Vec::new();
		sector.extend_from_slice(&((payload.len() + 1) as u32).to_be_bytes());
		sector.push(id);
		sector.extend_from_slice(payload);
		sector.resize(SECTOR_LEN, 0);
		file.extend(sector);
		file
	}

	#[test]
	fn reads_an_inline_zlib_chunk() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("r.0.0.mca");
		let payload = zlib(&chunk_nbt(0, 0, 3955, &[((0, 0, 0), "minecraft:stone")]));
		std::fs::write(&path, region_with_chunk(2, &payload)).unwrap();
		let region = RegionFile::open(&path).unwrap();
		let nbt = region.chunk_nbt(0, 0).unwrap().unwrap();
		assert_eq!(Chunk::parse(&nbt).unwrap().x, 0);
		assert!(region.chunk_nbt(1, 0).unwrap().is_none());
	}

	#[test]
	fn follows_external_chunks_stored_in_mcc_files() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("r.0.0.mca");
		std::fs::write(&path, region_with_chunk(128 + 2, &[])).unwrap();
		std::fs::write(dir.path().join("c.0.0.mcc"), zlib(&chunk_nbt(0, 0, 3955, &[]))).unwrap();
		let region = RegionFile::open(&path).unwrap();
		assert!(region.chunk_nbt(0, 0).unwrap().is_some());
	}

	#[test]
	fn parses_region_coordinates_from_the_file_name() {
		assert_eq!(parse_coords(Path::new("r.-3.12.mca")).unwrap(), (-3, 12));
		assert!(parse_coords(Path::new("chunks.mca")).is_err());
	}
}
