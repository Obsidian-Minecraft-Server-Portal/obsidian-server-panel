use std::io::Read;

use crate::error::{Result, WorldmapError};

/// Chunk compression ids from a region header. Ids with the high bit set mark externally stored chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
	Gzip,
	Zlib,
	Uncompressed,
	Lz4,
}

impl Compression {
	pub fn from_id(id: u8) -> Result<Self> {
		match id {
			1 => Ok(Self::Gzip),
			2 => Ok(Self::Zlib),
			3 => Ok(Self::Uncompressed),
			4 => Ok(Self::Lz4),
			other => Err(WorldmapError::Compression(other)),
		}
	}

	pub fn decode(self, data: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::Gzip => read_to_end(flate2::read::GzDecoder::new(data)),
			Self::Zlib => read_to_end(flate2::read::ZlibDecoder::new(data)),
			Self::Uncompressed => Ok(data.to_vec()),
			Self::Lz4 => super::lz4::decode_block_stream(data),
		}
	}
}

fn read_to_end(mut reader: impl Read) -> Result<Vec<u8>> {
	let mut out = Vec::new();
	reader.read_to_end(&mut out)?;
	Ok(out)
}
