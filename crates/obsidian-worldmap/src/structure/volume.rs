//! Volume records: a box of blocks with their states and light, and for sections their biomes. Little-endian. Layout: magic
//! `OBSV`, version, flags (bit 0: biomes follow), origin x, y, z as i32, size x, y, z as u8, the block palette (u32 state ids),
//! the packed palette indices, the sky light and the block light, each a mode byte and its values, then the biome palette
//! and cells when flagged. Voxels run x fastest, then z, then y.

use super::bits;
use crate::error::{Result, WorldmapError};

const MAGIC: [u8; 4] = *b"OBSV";
const VERSION: u16 = 1;
const FLAG_BIOMES: u16 = 1;
const LIGHT_UNIFORM: u8 = 0;
const LIGHT_NIBBLES: u8 = 1;
pub const BIOME_CELLS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Light {
    Uniform(u8),
    /// One nibble per voxel, low nibble first, so voxel `i` is byte `i / 2`.
    Nibbles(Vec<u8>),
}

impl Light {
    pub fn at(&self, index: usize) -> u8 {
        match self {
            Self::Uniform(level) => *level,
            Self::Nibbles(bytes) => {
                let byte = bytes[index / 2];
                if index.is_multiple_of(2) { byte & 0x0F } else { byte >> 4 }
            }
        }
    }

    /// Uniform when every voxel holds one level, so a flat array costs one byte.
    pub fn from_levels(levels: &[u8]) -> Self {
        match levels.first() {
            Some(&first) if levels.iter().all(|&level| level == first) => Self::Uniform(first),
            _ => Self::Nibbles(
                levels
                    .chunks(2)
                    .map(|pair| pair[0] | pair.get(1).map_or(0, |high| high << 4))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biomes {
    pub palette: Vec<u16>,
    /// One palette index per 4 by 4 by 4 cell, in the order `y / 4 * 16 + z / 4 * 4 + x / 4`.
    pub indices: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    pub origin: [i32; 3],
    pub size: [u8; 3],
    pub palette: Vec<u32>,
    pub indices: Vec<u16>,
    pub sky: Light,
    pub block: Light,
    pub biomes: Option<Biomes>,
}

impl Volume {
    pub fn voxels(&self) -> usize {
        self.size.iter().map(|&side| usize::from(side)).product()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        let flags = if self.biomes.is_some() { FLAG_BIOMES } else { 0 };
        out.extend_from_slice(&flags.to_le_bytes());
        for value in self.origin {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.extend_from_slice(&self.size);
        out.extend_from_slice(&(self.palette.len() as u16).to_le_bytes());
        for id in &self.palette {
            out.extend_from_slice(&id.to_le_bytes());
        }
        let width = bits::width_for(self.palette.len());
        out.push(width);
        out.extend(bits::pack(&self.indices, width));
        encode_light(&self.sky, &mut out);
        encode_light(&self.block, &mut out);
        if let Some(biomes) = &self.biomes {
            out.extend_from_slice(&(biomes.palette.len() as u16).to_le_bytes());
            for id in &biomes.palette {
                out.extend_from_slice(&id.to_le_bytes());
            }
            let width = bits::width_for(biomes.palette.len());
            out.push(width);
            out.extend(bits::pack(&biomes.indices, width));
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader { bytes };
        if reader.take(4)? != MAGIC {
            return Err(invalid("bad magic"));
        }
        if reader.u16()? != VERSION {
            return Err(invalid("unsupported version"));
        }
        let flags = reader.u16()?;
        let origin = [reader.i32()?, reader.i32()?, reader.i32()?];
        let size = [reader.u8()?, reader.u8()?, reader.u8()?];
        let voxels = size.iter().map(|&side| usize::from(side)).product::<usize>();
        if voxels == 0 {
            return Err(invalid("empty volume"));
        }
        let palette_len = usize::from(reader.u16()?);
        let palette = (0..palette_len).map(|_| reader.u32()).collect::<Result<Vec<u32>>>()?;
        let width = reader.u8()?;
        let indices = bits::unpack(reader.take(bits::byte_len(voxels, width))?, width, voxels)?;
        if palette_len == 0 || indices.iter().any(|&index| usize::from(index) >= palette_len) {
            return Err(invalid("palette index out of range"));
        }
        let sky = decode_light(&mut reader, voxels)?;
        let block = decode_light(&mut reader, voxels)?;
        let biomes = if flags & FLAG_BIOMES == 0 {
            None
        } else {
            let palette_len = usize::from(reader.u16()?);
            let palette = (0..palette_len).map(|_| reader.u16()).collect::<Result<Vec<u16>>>()?;
            let width = reader.u8()?;
            let indices = bits::unpack(reader.take(bits::byte_len(BIOME_CELLS, width))?, width, BIOME_CELLS)?;
            if palette_len == 0 || indices.iter().any(|&index| usize::from(index) >= palette_len) {
                return Err(invalid("biome index out of range"));
            }
            Some(Biomes { palette, indices })
        };
        Ok(Self {
            origin,
            size,
            palette,
            indices,
            sky,
            block,
            biomes,
        })
    }
}

fn encode_light(light: &Light, out: &mut Vec<u8>) {
    match light {
        Light::Uniform(level) => out.extend_from_slice(&[LIGHT_UNIFORM, *level]),
        Light::Nibbles(bytes) => {
            out.push(LIGHT_NIBBLES);
            out.extend_from_slice(bytes);
        }
    }
}

fn decode_light(reader: &mut Reader<'_>, voxels: usize) -> Result<Light> {
    match reader.u8()? {
        LIGHT_UNIFORM => Ok(Light::Uniform(reader.u8()?)),
        LIGHT_NIBBLES => Ok(Light::Nibbles(reader.take(voxels.div_ceil(2))?.to_vec())),
        _ => Err(invalid("unknown light mode")),
    }
}

fn invalid(reason: &str) -> WorldmapError {
    WorldmapError::Structure(reason.to_owned())
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        if self.bytes.len() < count {
            return Err(invalid("volume record is truncated"));
        }
        let (head, rest) = self.bytes.split_at(count);
        self.bytes = rest;
        Ok(head)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(self.u32()? as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_section_with_biomes_and_nibble_light() {
        let volume = Volume {
            origin: [-16, 32, 48],
            size: [16, 16, 16],
            palette: vec![0, 7, 9],
            indices: (0..4096_u16).map(|index| index % 3).collect(),
            sky: Light::Nibbles((0..2048_u16).map(|index| (index % 16) as u8 | 0xF0).collect()),
            block: Light::Uniform(0),
            biomes: Some(Biomes {
                palette: vec![0, 5],
                indices: (0..64_u16).map(|index| index % 2).collect(),
            }),
        };
        assert_eq!(Volume::decode(&volume.encode()).unwrap(), volume);
    }

    #[test]
    fn round_trips_a_single_voxel_border_without_biomes() {
        let volume = Volume {
            origin: [63, 0, 64],
            size: [1, 1, 1],
            palette: vec![0],
            indices: vec![0],
            sky: Light::Uniform(15),
            block: Light::Uniform(0),
            biomes: None,
        };
        assert_eq!(Volume::decode(&volume.encode()).unwrap(), volume);
    }

    #[test]
    fn collapses_flat_light_to_one_level() {
        assert_eq!(Light::from_levels(&[15; 9]), Light::Uniform(15));
        assert_eq!(Light::from_levels(&[1, 2, 3]).at(2), 3);
    }

    #[test]
    fn rejects_truncated_and_out_of_range_records() {
        let bytes = Volume {
            origin: [0; 3],
            size: [2, 1, 1],
            palette: vec![4],
            indices: vec![0, 0],
            sky: Light::Uniform(0),
            block: Light::Uniform(0),
            biomes: None,
        }
        .encode();
        assert!(Volume::decode(&bytes[..bytes.len() - 1]).is_err());
        assert!(Volume::decode(b"OBSM").is_err());
    }
}
