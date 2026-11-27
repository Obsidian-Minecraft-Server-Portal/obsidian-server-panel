// Faces of one cell, with the culling, ambient occlusion and light that the server's block tables describe.
// Corners and texture coordinates stay in the server's integer packing, so the output can be compared with it exactly.

import { tintFor } from "./biome-tint";
import { CELL_BLOCKS, type CellVoxels, PADDED, paddedIndex } from "./bundle";
import type { PreparedBlock, PreparedQuad, PreparedTable } from "./tables";

const PLANE = PADDED * PADDED;
/** Index step to the neighbour each direction points at: down, up, north, south, west, east. */
export const DIRECTION_STEP = [-PLANE, PLANE, -PADDED, PADDED, -1, 1];
/** The two axes (0 x, 1 y, 2 z) that run across each face. */
export const TANGENTS = [
  [0, 2],
  [0, 2],
  [0, 1],
  [0, 1],
  [1, 2],
  [1, 2],
];
const AXIS_STEP = [1, PLANE, PADDED];
const OCCLUSION = [Math.fround(0.4), Math.fround(0.6), Math.fround(0.8), 1];
const FULL_LIGHT = 15;
const ALL_FACES = 0b111111;
const WHITE = [255, 255, 255];

/** One visible face of a block, with the block's cell-local coordinates. */
export interface Face {
  x: number;
  y: number;
  z: number;
  quad: PreparedQuad;
  tint: number[];
  /** Brightness of each corner, 0 to 255. */
  light: number[];
}

export interface EmittedQuad {
  /** Chunk x, section y and chunk z of the block the quad belongs to. */
  section: [number, number, number];
  layer: number;
  /** Corners as x, y, z in 1/256 of a block, relative to the section origin. */
  corners: number[];
  /** Atlas coordinates, two per corner, as u16. */
  uvs: number[];
  direction: number;
  tint: number[];
  light: number[];
}

export function meshCell(voxels: CellVoxels, table: PreparedTable): Face[] {
  const faces: Face[] = [];
  for (let y = 0; y < CELL_BLOCKS; y++) {
    for (let z = 0; z < CELL_BLOCKS; z++) {
      for (let x = 0; x < CELL_BLOCKS; x++) {
        const index = paddedIndex(x, y, z);
        const block = table.blocks[voxels.ids[index]];
        if (block.air) {
          continue;
        }
        const biome = table.biomes[voxels.biomes[(y * CELL_BLOCKS + z) * CELL_BLOCKS + x]];
        for (const quad of block.quads) {
          if (quad.cullface >= 0 && isCulled(voxels, table.blocks, block, quad, index)) {
            continue;
          }
          const tint = quad.colour ?? (quad.tint < 0 ? WHITE : tintFor(quad.tint, biome, cellX(voxels) + x, cellZ(voxels) + z));
          const light = quad.ao
            ? cornerLight(voxels, table.blocks, index, quad, table.ambientLight)
            : flatLight(voxels, index, quad, table.ambientLight);
          faces.push({ x, y, z, quad, tint, light });
        }
      }
    }
  }
  return faces;
}

export function emit(face: Face, cell: [number, number, number]): EmittedQuad {
  const absoluteX = cell[0] * CELL_BLOCKS + face.x;
  const absoluteY = cell[1] * CELL_BLOCKS + face.y;
  const absoluteZ = cell[2] * CELL_BLOCKS + face.z;
  return {
    section: [Math.floor(absoluteX / 16), Math.floor(absoluteY / 16), Math.floor(absoluteZ / 16)],
    layer: face.quad.layer,
    corners: positions(face.quad, face.x & 15, face.y & 15, face.z & 15),
    uvs: Array.from(face.quad.uvs),
    direction: face.quad.direction,
    tint: face.tint,
    light: face.light,
  };
}

function cellX(voxels: CellVoxels): number {
  return voxels.cell[0] * CELL_BLOCKS;
}

function cellZ(voxels: CellVoxels): number {
  return voxels.cell[2] * CELL_BLOCKS;
}

function isCulled(voxels: CellVoxels, blocks: PreparedBlock[], block: PreparedBlock, quad: PreparedQuad, index: number): boolean {
  const neighbour = blocks[voxels.ids[index + DIRECTION_STEP[quad.cullface]]];
  if (neighbour.air) {
    return false;
  }
  if (neighbour.name === block.name && (block.fluid !== 0 || block.skipSame)) {
    return true;
  }
  return ((neighbour.full >> (quad.cullface ^ 1)) & 1) === 1;
}

function positions(quad: PreparedQuad, localX: number, localY: number, localZ: number): number[] {
  const local = [localX, localY, localZ];
  const out: number[] = [];
  for (let corner = 0; corner < 4; corner++) {
    for (let axis = 0; axis < 3; axis++) {
      const offset = Math.fround(quad.corners[corner * 3 + axis] / 16);
      out.push(Math.round(Math.fround(Math.fround(local[axis] + offset) * 256)));
    }
  }
  return out;
}

function cornerLight(voxels: CellVoxels, blocks: PreparedBlock[], index: number, quad: PreparedQuad, ambient: number): number[] {
  const front = index + DIRECTION_STEP[quad.direction];
  const [first, second] = TANGENTS[quad.direction];
  const light: number[] = [0, 0, 0, 0];
  for (let corner = 0; corner < 4; corner++) {
    const firstSide = quad.corners[corner * 3 + first] < 8 ? -1 : 1;
    const secondSide = quad.corners[corner * 3 + second] < 8 ? -1 : 1;
    const besideFirst = front + firstSide * AXIS_STEP[first];
    const besideSecond = front + secondSide * AXIS_STEP[second];
    const diagonal = besideFirst + secondSide * AXIS_STEP[second];
    const occludedFirst = isOccluder(voxels, blocks, besideFirst);
    const occludedSecond = isOccluder(voxels, blocks, besideSecond);
    const occlusion =
      occludedFirst && occludedSecond
        ? 0
        : 3 - (Number(occludedFirst) + Number(occludedSecond) + Number(isOccluder(voxels, blocks, diagonal)));
    const level =
      (lightLevel(voxels, front) + lightLevel(voxels, besideFirst) + lightLevel(voxels, besideSecond) + lightLevel(voxels, diagonal)) / 4;
    light[corner] = Math.round(Math.fround(Math.fround(OCCLUSION[occlusion] * lightCurve(level, ambient)) * 255));
  }
  return light;
}

/** Faces of models without ambient occlusion take one brightness, from the cell their cullface points at or from their own. */
function flatLight(voxels: CellVoxels, index: number, quad: PreparedQuad, ambient: number): number[] {
  const source = quad.cullface >= 0 ? index + DIRECTION_STEP[quad.cullface] : index;
  const value = Math.round(Math.fround(lightCurve(lightLevel(voxels, source), ambient) * 255));
  return [value, value, value, value];
}

function isOccluder(voxels: CellVoxels, blocks: PreparedBlock[], index: number): boolean {
  const block = blocks[voxels.ids[index]];
  return !block.air && block.full === ALL_FACES;
}

function lightLevel(voxels: CellVoxels, index: number): number {
  return Math.max(voxels.sky[index], voxels.block[index]);
}

function lightCurve(level: number, ambient: number): number {
  const t = Math.min(Math.max(Math.fround(level / FULL_LIGHT), 0), 1);
  const numerator = Math.fround(Math.fround(1 - ambient) * t);
  const denominator = Math.fround(4 - Math.fround(3 * t));
  return Math.fround(ambient + Math.fround(numerator / denominator));
}
