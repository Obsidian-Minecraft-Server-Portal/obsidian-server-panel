//! lz4-java block streams, the LZ4 chunk payload Minecraft writes. Each block has a 21-byte header.
//! The per-block checksum is not verified; corrupt data fails to decompress or to parse instead.

use crate::error::{Result, WorldmapError};

const MAGIC: &[u8; 8] = b"LZ4Block";
const HEADER_LEN: usize = 21;
const METHOD_RAW: u8 = 0x10;
const METHOD_LZ4: u8 = 0x20;

pub fn decode_block_stream(mut input: &[u8]) -> Result<Vec<u8>> {
	let mut out = Vec::new();
	while !input.is_empty() {
		let header = input
			.get(..HEADER_LEN)
			.filter(|header| header.starts_with(MAGIC))
			.ok_or_else(|| WorldmapError::Chunk("LZ4 block header is missing or truncated".into()))?;
		let method = header[8] & 0xF0;
		let compressed = le_u32(&header[9..13]);
		let decompressed = le_u32(&header[13..17]);
		let body = input
			.get(HEADER_LEN..HEADER_LEN + compressed)
			.ok_or_else(|| WorldmapError::Chunk("LZ4 block body is truncated".into()))?;
		match method {
			METHOD_RAW => out.extend_from_slice(body),
			METHOD_LZ4 => out.extend(
				lz4_flex::block::decompress(body, decompressed)
					.map_err(|error| WorldmapError::Chunk(format!("LZ4 block: {error}")))?,
			),
			other => return Err(WorldmapError::Chunk(format!("unknown LZ4 block method {other:#04x}"))),
		}
		input = &input[HEADER_LEN + compressed..];
	}
	Ok(out)
}

fn le_u32(bytes: &[u8]) -> usize {
	u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
}

#[cfg(test)]
mod tests {
	use super::*;

	fn block(method: u8, body: &[u8], decompressed: usize) -> Vec<u8> {
		let mut out = MAGIC.to_vec();
		out.push(method);
		out.extend_from_slice(&(body.len() as u32).to_le_bytes());
		out.extend_from_slice(&(decompressed as u32).to_le_bytes());
		out.extend_from_slice(&[0; 4]);
		out.extend_from_slice(body);
		out
	}

	#[test]
	fn decodes_lz4_and_raw_blocks_in_sequence() {
		let text = b"minecraft chunk minecraft chunk minecraft chunk";
		let mut stream = block(METHOD_LZ4, &lz4_flex::block::compress(text), text.len());
		stream.extend(block(METHOD_RAW, b"raw", 3));
		let expected = [&text[..], &b"raw"[..]].concat();
		assert_eq!(decode_block_stream(&stream).unwrap(), expected);
	}

	#[test]
	fn rejects_streams_without_the_block_magic() {
		assert!(decode_block_stream(&[0_u8; 21]).is_err());
	}
}
