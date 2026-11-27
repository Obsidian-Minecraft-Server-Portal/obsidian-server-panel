// Decodes the cell bundles the server writes (crates/obsidian-worldmap/src/structure/cell.rs and volume.rs) into padded voxel
// arrays. Everything is little-endian.

import { StaleTablesError } from "./tables";

/** A cell is 64 blocks a side. The arrays hold one more block on each side, so faces can read their neighbours. */
export const CELL_BLOCKS = 64;
export const PADDED = CELL_BLOCKS + 2;
const PADDED_LAYER = PADDED * PADDED;
const PADDED_VOXELS = PADDED_LAYER * PADDED;
const OPEN_SKY = 15;
const BUNDLE_HEADER = 24;

export interface CellVoxels {
  /** Cell coordinates, in units of 64 blocks. */
  cell: [number, number, number];
  /** State id per voxel, index `(y + 1) * 66 * 66 + (z + 1) * 66 + (x + 1)` for cell-local `x`, `y`, `z` from -1 to 64. */
  ids: Uint32Array;
  sky: Uint8Array;
  block: Uint8Array;
  /** Biome id per block of the cell, index `(y * 64 + z) * 64 + x`. */
  biomes: Uint16Array;
}

export function paddedIndex(x: number, y: number, z: number): number {
  return (y + 1) * PADDED_LAYER + (z + 1) * PADDED + (x + 1);
}

/** How many block states and biomes the loaded tables hold. A bundle may only name ids below these. */
export interface TableLimits {
  blocks: number;
  biomes: number;
}

export function paintCell(bundle: ArrayBuffer, limits: TableLimits): CellVoxels {
  const bytes = new Uint8Array(bundle);
  const view = new DataView(bundle);
  if (readMagic(bytes) !== "OBSB" || view.getUint16(4, true) !== 1) {
    throw new Error("cell bundle has a bad header");
  }
  const voxels: CellVoxels = {
    cell: [view.getInt32(8, true), view.getInt32(12, true), view.getInt32(16, true)],
    ids: new Uint32Array(PADDED_VOXELS),
    sky: new Uint8Array(PADDED_VOXELS).fill(OPEN_SKY),
    block: new Uint8Array(PADDED_VOXELS),
    biomes: new Uint16Array(CELL_BLOCKS * CELL_BLOCKS * CELL_BLOCKS),
  };
  const count = view.getUint32(20, true);
  let at = BUNDLE_HEADER;
  for (let record = 0; record < count; record++) {
    const length = view.getUint32(at, true);
    paintVolume(bytes.subarray(at + 4, at + 4 + length), voxels, limits);
    at += 4 + length;
  }
  return voxels;
}

function paintVolume(bytes: Uint8Array, voxels: CellVoxels, limits: TableLimits): void {
  if (readMagic(bytes) !== "OBSV") {
    throw new Error("volume record has a bad magic");
  }
  const reader = new Reader(bytes);
  reader.skip(4);
  reader.u16();
  const flags = reader.u16();
  const origin = [reader.i32(), reader.i32(), reader.i32()];
  const size = [reader.u8(), reader.u8(), reader.u8()];
  const voxelCount = size[0] * size[1] * size[2];
  const palette = new Uint32Array(reader.u16());
  for (let index = 0; index < palette.length; index++) {
    palette[index] = reader.u32();
  }
  if (palette.some((id) => id >= limits.blocks)) {
    throw new StaleTablesError(`a volume names a block state past the ${limits.blocks} the tables hold`);
  }
  const width = reader.u8();
  const indices = unpackIndices(reader.take(packedBytes(voxelCount, width)), voxelCount, width);
  const sky = reader.light(voxelCount);
  const block = reader.light(voxelCount);
  const biomes = flags & 1 ? readBiomes(reader) : undefined;
  if (biomes && biomes.palette.some((id) => id >= limits.biomes)) {
    throw new StaleTablesError(`a volume names a biome past the ${limits.biomes} the tables hold`);
  }

  const [cellX, cellY, cellZ] = voxels.cell.map((coordinate) => coordinate * CELL_BLOCKS);
  for (let y = 0; y < size[1]; y++) {
    for (let z = 0; z < size[2]; z++) {
      for (let x = 0; x < size[0]; x++) {
        const voxel = x + size[0] * (z + size[2] * y);
        const local = [origin[0] + x - cellX, origin[1] + y - cellY, origin[2] + z - cellZ];
        if (local.some((coordinate) => coordinate < -1 || coordinate > CELL_BLOCKS)) {
          continue;
        }
        const index = paddedIndex(local[0], local[1], local[2]);
        voxels.ids[index] = palette[indices[voxel]];
        voxels.sky[index] = sky.at(voxel);
        voxels.block[index] = block.at(voxel);
        if (biomes && local.every((coordinate) => coordinate >= 0 && coordinate < CELL_BLOCKS)) {
          const cellIndex = (y >> 2) * 16 + (z >> 2) * 4 + (x >> 2);
          voxels.biomes[(local[1] * CELL_BLOCKS + local[2]) * CELL_BLOCKS + local[0]] = biomes.palette[biomes.indices[cellIndex]];
        }
      }
    }
  }
}

interface Biomes {
  palette: Uint16Array;
  indices: Uint16Array;
}

function readBiomes(reader: Reader): Biomes {
  const palette = new Uint16Array(reader.u16());
  for (let index = 0; index < palette.length; index++) {
    palette[index] = reader.u16();
  }
  const width = reader.u8();
  return { palette, indices: unpackIndices(reader.take(packedBytes(64, width)), 64, width) };
}

/** Palette indices packed at 0, 1, 2, 4, 8 or 16 bits per entry, least significant bits first, never straddling a byte. */
function unpackIndices(bytes: Uint8Array, count: number, width: number): Uint16Array {
  const indices = new Uint16Array(count);
  if (width === 0) {
    return indices;
  }
  if (width === 16) {
    for (let index = 0; index < count; index++) {
      indices[index] = bytes[index * 2] | (bytes[index * 2 + 1] << 8);
    }
    return indices;
  }
  const perByte = 8 / width;
  const mask = (1 << width) - 1;
  for (let index = 0; index < count; index++) {
    indices[index] = (bytes[Math.floor(index / perByte)] >> ((index % perByte) * width)) & mask;
  }
  return indices;
}

function packedBytes(count: number, width: number): number {
  return Math.ceil((count * width) / 8);
}

function readMagic(bytes: Uint8Array): string {
  return String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
}

/** A light array: one level for every voxel, or one nibble per voxel, low nibble first. */
class Light {
  constructor(
    private readonly uniform: number | undefined,
    private readonly nibbles: Uint8Array | undefined,
  ) {}

  at(voxel: number): number {
    if (this.uniform !== undefined) {
      return this.uniform;
    }
    const byte = this.nibbles![voxel >> 1];
    return voxel & 1 ? byte >> 4 : byte & 0x0f;
  }
}

class Reader {
  private at = 0;

  constructor(private readonly bytes: Uint8Array) {}

  skip(count: number): void {
    this.at += count;
  }

  take(count: number): Uint8Array {
    const slice = this.bytes.subarray(this.at, this.at + count);
    this.at += count;
    return slice;
  }

  u8(): number {
    return this.bytes[this.at++];
  }

  u16(): number {
    const value = this.bytes[this.at] | (this.bytes[this.at + 1] << 8);
    this.at += 2;
    return value;
  }

  u32(): number {
    const value = new DataView(this.bytes.buffer, this.bytes.byteOffset + this.at, 4).getUint32(0, true);
    this.at += 4;
    return value;
  }

  i32(): number {
    return this.u32() | 0;
  }

  light(voxelCount: number): Light {
    if (this.u8() === 0) {
      return new Light(this.u8(), undefined);
    }
    return new Light(undefined, this.take(Math.ceil(voxelCount / 2)));
  }
}
