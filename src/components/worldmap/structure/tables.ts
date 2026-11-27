// The block and biome tables the server writes beside each dimension's structure (blocks.json and biomes.json).

/** A bundle names a block state or a biome that the loaded tables do not hold, because the tables were fetched before the render. */
export class StaleTablesError extends Error {}

export type GrassModifier = "none" | "darkForest" | "swamp";

export interface QuadJson {
  corners: number[];
  uvs: number[];
  cell: [number, number];
  direction: number;
  cullface: number;
  /** Direction index of the shade, or -1 for full brightness. */
  shade: number;
  /** 0 for grass, 1 for foliage, 2 for water, 3 for dry foliage, or -1 for none. */
  tint: number;
  /** The fixed colour the block state carries, in place of a biome tint. */
  colour: [number, number, number] | null;
  ao: boolean;
  layer: number;
}

export interface BlockJson {
  name: number;
  air: boolean;
  fluid: number;
  full: number;
  skipSame: boolean;
  quads: QuadJson[];
}

export interface BlockTableJson {
  atlasWidth: number;
  atlasHeight: number;
  ambientLight: number;
  names: string[];
  states: BlockJson[];
}

export interface BiomeJson {
  name: string;
  grass: [number, number, number];
  foliage: [number, number, number];
  dryFoliage: [number, number, number];
  water: [number, number, number];
  grassModifier: GrassModifier;
}

/** The shade index of a face drawn at full brightness, one past the six directions. */
export const UNSHADED = 6;

/** A quad as the mesher reads it. Corner values are f32, and texture coordinates are the atlas values the server packs. */
export interface PreparedQuad {
  corners: Float32Array;
  uvs: Uint16Array;
  direction: number;
  cullface: number;
  /** Index into the shade table: a direction, or `UNSHADED`. */
  shade: number;
  tint: number;
  colour: [number, number, number] | null;
  ao: boolean;
  layer: number;
  /** Texel coordinates, 0 to 16, two per corner, before the atlas offset. */
  texel: Float32Array;
  /** Which of the eight square symmetries maps the face to its texel coordinates, or -1 when none does. */
  pattern: number;
  /** Top left texel of the quad's texture cell in the atlas. */
  cell: [number, number];
  /** Whether the quad covers its block face exactly, so faces like it can merge into larger rectangles. */
  unit: boolean;
}

export interface PreparedBlock {
  air: boolean;
  fluid: number;
  full: number;
  skipSame: boolean;
  name: number;
  quads: PreparedQuad[];
}

export interface PreparedTable {
  blocks: PreparedBlock[];
  biomes: BiomeJson[];
  atlasWidth: number;
  atlasHeight: number;
  ambientLight: number;
}

export function prepareTable(blocks: BlockTableJson, biomes: BiomeJson[]): PreparedTable {
  return {
    blocks: blocks.states.map((state) => ({
      air: state.air,
      fluid: state.fluid,
      full: state.full,
      skipSame: state.skipSame,
      name: state.name,
      quads: state.quads.map((quad) => ({
        corners: Float32Array.from(quad.corners),
        uvs: encodeUvs(quad.uvs, quad.cell, blocks.atlasWidth, blocks.atlasHeight),
        direction: quad.direction,
        cullface: quad.cullface,
        shade: quad.shade < 0 ? UNSHADED : quad.shade,
        tint: quad.tint,
        colour: quad.colour,
        ao: quad.ao,
        layer: quad.layer,
        texel: Float32Array.from(quad.uvs),
        pattern: symmetryOf(Float32Array.from(quad.corners), Float32Array.from(quad.uvs), quad.direction),
        cell: [quad.cell[0], quad.cell[1]],
        unit: coversBlockFace(Float32Array.from(quad.corners), quad.direction),
      })),
    })),
    biomes,
    atlasWidth: blocks.atlasWidth,
    atlasHeight: blocks.atlasHeight,
    ambientLight: Math.fround(blocks.ambientLight),
  };
}

/** The axis and boundary of each face's plane: down, up, north, south, west, east, as the server's `covers_block_face`. */
const PLANE_AXIS = [1, 1, 2, 2, 0, 0];
const PLANE_BOUNDARY = [0, 16, 0, 16, 0, 16];

function coversBlockFace(corners: Float32Array, direction: number): boolean {
  const axis = PLANE_AXIS[direction];
  for (let corner = 0; corner < 4; corner++) {
    if (Math.abs(corners[corner * 3 + axis] - PLANE_BOUNDARY[direction]) > 1e-3) {
      return false;
    }
  }
  return [0, 1, 2]
    .filter((other) => other !== axis)
    .every((other) => {
      const values = [0, 1, 2, 3].map((corner) => corners[corner * 3 + other]);
      return Math.abs(Math.min(...values)) < 1e-3 && Math.abs(Math.max(...values) - 16) < 1e-3;
    });
}

/** Atlas coordinates of texel positions, rounded the way the server rounds them to u16. */
function encodeUvs(uvs: number[], cell: [number, number], width: number, height: number): Uint16Array {
  const encoded = new Uint16Array(8);
  for (let corner = 0; corner < 4; corner++) {
    const u = atlasCoordinate(cell[0], uvs[corner * 2], width);
    const v = atlasCoordinate(cell[1], uvs[corner * 2 + 1], height);
    encoded[corner * 2] = encodeUnit(u);
    encoded[corner * 2 + 1] = encodeUnit(v);
  }
  return encoded;
}

function atlasCoordinate(cellOrigin: number, texel: number, size: number): number {
  const clamped = Math.min(Math.max(Math.fround(texel), 0), 16);
  return Math.fround(Math.fround(cellOrigin + clamped) / Math.fround(size));
}

function encodeUnit(value: number): number {
  return Math.round(Math.fround(Math.min(Math.max(value, 0), 1) * 65535));
}

const SYMMETRIES: ((s: number, t: number) => [number, number])[] = [
  (s, t) => [s, t],
  (s, t) => [1 - s, t],
  (s, t) => [s, 1 - t],
  (s, t) => [1 - s, 1 - t],
  (s, t) => [t, s],
  (s, t) => [1 - t, s],
  (s, t) => [t, 1 - s],
  (s, t) => [1 - t, 1 - s],
];

/**
 * The symmetry that maps a face's corners to its texel coordinates, so a face covering its block can draw any block-local
 * point from the texture without storing per-corner coordinates. Faces with partial texture coordinates have none.
 */
function symmetryOf(corners: Float32Array, texel: Float32Array, direction: number): number {
  const [first, second] = TANGENT_AXES[direction];
  return SYMMETRIES.findIndex((map) =>
    [0, 1, 2, 3].every((corner) => {
      const s = corners[corner * 3 + first] > 8 ? 1 : 0;
      const t = corners[corner * 3 + second] > 8 ? 1 : 0;
      const [u, v] = map(s, t);
      return Math.abs(texel[corner * 2] - 16 * u) < 1e-3 && Math.abs(texel[corner * 2 + 1] - 16 * v) < 1e-3;
    }),
  );
}

const TANGENT_AXES = [
  [0, 2],
  [0, 2],
  [0, 1],
  [0, 1],
  [1, 2],
  [1, 2],
];
