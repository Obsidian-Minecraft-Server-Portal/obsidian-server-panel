// Faces of one cell, lit and culled the way the game draws them: smooth lighting from the blocks around each corner, flat lighting
// for models without ambient occlusion, the brightness of each face direction, and the heights and culling of fluids and bars.
// Corners and texture coordinates stay in the server's integer packing, so the output can be compared with it exactly.

import { tintFor } from "./biome-tint";
import { CELL_BLOCKS, type CellVoxels, PADDED, paddedIndex } from "./bundle";
import { type PreparedBlock, type PreparedQuad, type PreparedTable, UNSHADED } from "./tables";

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
/** The directions of each axis's low and high side: west and east, down and up, north and south. */
const AXIS_SIDES = [
  [4, 5],
  [0, 1],
  [2, 3],
];
/** The four blocks a face's ambient occlusion samples beside it, in the game's order: two opposite pairs. */
const OCCLUSION_NEIGHBOURS = [
  [4, 5, 2, 3],
  [5, 4, 2, 3],
  [1, 0, 5, 4],
  [4, 5, 0, 1],
  [1, 0, 2, 3],
  [0, 1, 2, 3],
];
const DOWN = 0;
const UP = 1;
const AIR_STATE = 0;
const OPEN_SKY = 15;
const FULL_LIGHT = 15;
const FULL_BLOCK_FACES = 0b111111;
/** The shade a full-block neighbour gives a corner. Other blocks leave the corner bright. */
const FULL_BLOCK_SHADE = 0.2;
const BLOCK_HEIGHT = 16;
const SOURCE_AMOUNT = 8;
const FLUID_STEPS = 9;
const WHITE = [255, 255, 255];

/** One visible face of a block, with the block's cell-local coordinates. */
export interface Face {
  x: number;
  y: number;
  z: number;
  quad: PreparedQuad;
  /** The quad's corners, lowered to the fluid's surface where the quad is part of one. */
  corners: Float32Array;
  tint: number[];
  /** Brightness of each corner, 0 to 255, with the face's directional brightness already in it. */
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
        const height = fluidHeight(voxels, table.blocks, index);
        for (const quad of block.quads) {
          if (quad.cullface >= 0 && isCulled(voxels, table.blocks, block, quad, index)) {
            continue;
          }
          const tint = quad.colour ?? (quad.tint < 0 ? WHITE : tintFor(quad.tint, biome, cellX(voxels) + x, cellZ(voxels) + z));
          faces.push({
            x,
            y,
            z,
            quad,
            corners: quad.fluid ? lowered(quad.corners, height) : quad.corners,
            tint,
            light: quad.ao ? smoothLight(voxels, table, index, block, quad) : flatLight(voxels, table, index, block, quad),
          });
        }
      }
    }
  }
  return faces;
}

function cellX(voxels: CellVoxels): number {
  return voxels.cell[0] * CELL_BLOCKS;
}

function cellZ(voxels: CellVoxels): number {
  return voxels.cell[2] * CELL_BLOCKS;
}

/** The block at a padded index. Positions past the border the bundle carries read as air. */
function blockAt(voxels: CellVoxels, blocks: PreparedBlock[], at: number): PreparedBlock {
  return blocks[voxels.ids[at] ?? AIR_STATE];
}

function isCulled(voxels: CellVoxels, blocks: PreparedBlock[], block: PreparedBlock, quad: PreparedQuad, index: number): boolean {
  const neighbourIndex = index + DIRECTION_STEP[quad.cullface];
  const neighbour = blockAt(voxels, blocks, neighbourIndex);
  if (neighbour.air) {
    return false;
  }
  if (quad.fluid && block.fluid !== 0 && neighbour.fluid === block.fluid && fluidHidden(voxels, blocks, quad.cullface, index, neighbourIndex)) {
    return true;
  }
  if (block.skip === 1 && neighbour.name === block.name) {
    return true;
  }
  if (block.bars && neighbour.bars && joinsBars(block, neighbour, quad.cullface)) {
    return true;
  }
  return ((neighbour.full >> (quad.cullface ^ 1)) & 1) === 1;
}

/** Iron bars and panes leave out a face that meets a neighbouring bar: a side only where both connect across it, and a top or
 * bottom only against the same block. */
function joinsBars(block: PreparedBlock, neighbour: PreparedBlock, face: number): boolean {
  if (face === DOWN || face === UP) {
    return neighbour.name === block.name;
  }
  return ((block.connections >> face) & 1) === 1 && ((neighbour.connections >> (face ^ 1)) & 1) === 1;
}

/** A fluid face inside a column of the same fluid is hidden. A side beside a neighbour of the same fluid shows only where the
 * neighbour's surface is lower. */
function fluidHidden(voxels: CellVoxels, blocks: PreparedBlock[], face: number, index: number, neighbourIndex: number): boolean {
  if (face === UP || face === DOWN) {
    return true;
  }
  return fluidHeight(voxels, blocks, neighbourIndex) >= fluidHeight(voxels, blocks, index);
}

/** The height of a fluid's surface in model units: a full block inside a column of the same fluid, else the fluid's amount. */
function fluidHeight(voxels: CellVoxels, blocks: PreparedBlock[], index: number): number {
  const block = blockAt(voxels, blocks, index);
  if (block.fluid === 0) {
    return 0;
  }
  if (blockAt(voxels, blocks, index + PLANE).fluid === block.fluid) {
    return BLOCK_HEIGHT;
  }
  const amount = block.fluidLevel >= 8 ? SOURCE_AMOUNT : SOURCE_AMOUNT - block.fluidLevel;
  return (BLOCK_HEIGHT * amount) / FLUID_STEPS;
}

function lowered(corners: Float32Array, height: number): Float32Array {
  const result = corners.slice();
  for (let component = 1; component < result.length; component += 3) {
    if (result[component] > BLOCK_HEIGHT - 0.5) {
      result[component] = height;
    }
  }
  return result;
}

/** The smooth lighting of a face, as the game computes it. Each corner averages the shade of the two blocks beside it, the block
 * diagonal to it and the block the face's own position is in, and blends their light the same way. */
function smoothLight(voxels: CellVoxels, table: PreparedTable, index: number, block: PreparedBlock, quad: PreparedQuad): number[] {
  const out = DIRECTION_STEP[quad.direction];
  const cubic = isCubic(quad, block);
  const base = cubic ? index + out : index;
  const centre = cubic || !isOpaque(voxels, table.blocks, index + out) ? lightCoords(voxels, index + out) : lightCoords(voxels, index);
  const ownShade = shadeAt(voxels, table.blocks, base);
  const [first, second] = TANGENTS[quad.direction];
  const neighbours = OCCLUSION_NEIGHBOURS[quad.direction];
  const directional = directionalFactor(table, quad);
  const light: number[] = [0, 0, 0, 0];
  for (let corner = 0; corner < 4; corner++) {
    const alongFirst = DIRECTION_STEP[AXIS_SIDES[first][quad.corners[corner * 3 + first] < 8 ? 0 : 1]];
    const alongSecond = DIRECTION_STEP[AXIS_SIDES[second][quad.corners[corner * 3 + second] < 8 ? 0 : 1]];
    const sideFirst = base + alongFirst;
    const sideSecond = base + alongSecond;
    const open = !isOpaque(voxels, table.blocks, sideFirst + out) || !isOpaque(voxels, table.blocks, sideSecond + out);
    const diagonal = open ? sideFirst + alongSecond : base + DIRECTION_STEP[neighbours[0]];
    const shadeValue =
      (shadeAt(voxels, table.blocks, sideFirst) + shadeAt(voxels, table.blocks, sideSecond) + shadeAt(voxels, table.blocks, diagonal) + ownShade) / 4;
    const blended = smoothBlend(lightCoords(voxels, sideFirst), lightCoords(voxels, sideSecond), lightCoords(voxels, diagonal), centre);
    light[corner] = clampByte(255 * shadeValue * directional * lightCurve(Math.max(sky(blended), blockLevel(blended)), table.ambientLight));
  }
  return light;
}

/** The brightness of a model without ambient occlusion: one level for each face, from the block the face looks at. */
function flatLight(voxels: CellVoxels, table: PreparedTable, index: number, block: PreparedBlock, quad: PreparedQuad): number[] {
  const source = isCubic(quad, block) ? index + DIRECTION_STEP[quad.direction] : index;
  const level = Math.max(skyAt(voxels, source), blockLightAt(voxels, source));
  const value = clampByte(255 * directionalFactor(table, quad) * lightCurve(level, table.ambientLight));
  return [value, value, value, value];
}

/** A cube's face: flat on its block's boundary, or flat against a block with a full collision. */
function isCubic(quad: PreparedQuad, block: PreparedBlock): boolean {
  return quad.planar && (quad.onBoundary || block.collisionFull);
}

function shadeAt(voxels: CellVoxels, blocks: PreparedBlock[], at: number): number {
  return blockAt(voxels, blocks, at).shadeFull ? FULL_BLOCK_SHADE : 1;
}

/** Whether a block blocks light through it, as a solid block does. */
function isOpaque(voxels: CellVoxels, blocks: PreparedBlock[], at: number): boolean {
  return blockAt(voxels, blocks, at).full === FULL_BLOCK_FACES;
}

/** Unshaded faces take the brightness of the up face, as the game shades faces it cannot orient. */
function directionalFactor(table: PreparedTable, quad: PreparedQuad): number {
  return table.cardinal[quad.shade === UNSHADED ? UP : quad.shade];
}

function skyAt(voxels: CellVoxels, at: number): number {
  return voxels.sky[at] ?? OPEN_SKY;
}

function blockLightAt(voxels: CellVoxels, at: number): number {
  return voxels.block[at] ?? 0;
}

/** The packed light of a cell, in the game's layout: block light in bits 4 to 7 and sky light in bits 20 to 23. */
function lightCoords(voxels: CellVoxels, at: number): number {
  return (blockLightAt(voxels, at) << 4) | (skyAt(voxels, at) << 20);
}

function sky(packed: number): number {
  return (packed >> 20) & 0xf;
}

function blockLevel(packed: number): number {
  return (packed >> 4) & 0xf;
}

/** The game's blend of four packed light values. A sample with no sky or block light takes the centre's, so a dark neighbour does
 * not darken a corner the centre lights. */
function smoothBlend(a: number, b: number, c: number, centre: number): number {
  const lit = sky(centre) > 2 || blockLevel(centre) > 2;
  const total = (lit ? replaceDark(a, centre) : a) + (lit ? replaceDark(b, centre) : b) + (lit ? replaceDark(c, centre) : c) + centre;
  return (total >> 2) & 0x00ff00ff;
}

function replaceDark(value: number, centre: number): number {
  if (value === 0) {
    return centre;
  }
  return sky(value) === 0 ? value | (centre & 0xff0000) : value;
}

function lightCurve(level: number, ambient: number): number {
  const t = Math.min(Math.max(Math.fround(level / FULL_LIGHT), 0), 1);
  const numerator = Math.fround(Math.fround(1 - ambient) * t);
  const denominator = Math.fround(4 - Math.fround(3 * t));
  return Math.fround(ambient + Math.fround(numerator / denominator));
}

function clampByte(value: number): number {
  return Math.min(255, Math.max(0, Math.round(value)));
}
