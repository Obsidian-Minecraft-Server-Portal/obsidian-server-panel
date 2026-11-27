// A cell's faces as the browser draws them. Unit faces, merged or not, become instances that a shader expands; the rest become
// explicit triangles, shaded the way the explicit path always was. Layers are opaque, cutout and translucent.

import { mergeFaces } from "./cell-greedy";
import { type Face, TANGENTS } from "./cell-mesh";
import type { PreparedQuad, PreparedTable } from "./tables";

/** Vanilla's directional brightness: down, up, north, south, west, east, then full brightness for unshaded faces. */
export const FACE_SHADE = [0.5, 1, 0.8, 0.8, 0.6, 0.6, 1];
/** Triangles of a quad split along its corner 0 to 2 diagonal, or its 1 to 3 diagonal, in quad order. */
const ZERO_TWO = [0, 1, 2, 0, 2, 3];
const ONE_THREE = [0, 1, 3, 1, 2, 3];
/** Texel coordinates stop half a texel inside each texture cell, so sampling never reaches the neighbour. */
const INSET_TEXELS = 0.5;
const LAYERS = 3;

export interface InstanceArrays {
  count: number;
  /** The block an instance starts at, cell-local, three per instance. */
  origin: Uint8Array;
  /** Blocks along the first and second tangent axes of the face, two per instance. */
  size: Uint8Array;
  direction: Uint8Array;
  /** The shade index of each instance, as `FACE_SHADE`. */
  shade: Uint8Array;
  /** The texture symmetry (0 to 7), plus 8 when the faces split along the template's 0 to 2 diagonal. */
  pattern: Uint8Array;
  /** The top left texel of the texture cell in the atlas, two per instance. */
  cell: Uint16Array;
  tint: Uint8Array;
  /** Corner brightness in template order, (0, 0), (1, 0), (1, 1), (0, 1), four per instance. */
  light: Uint8Array;
}

export interface ExplicitArrays {
  quads: number;
  /** Cell-local block units, twelve per quad. */
  positions: Float32Array;
  /** Atlas coordinates with the half-texel inset, eight per quad. */
  uvs: Float32Array;
  colours: Float32Array;
  indices: Uint32Array;
}

export interface LayerArrays {
  instances: InstanceArrays;
  explicit: ExplicitArrays;
}

export function buildLayers(faces: Face[], table: PreparedTable): LayerArrays[] {
  const builders = Array.from({ length: LAYERS }, () => new LayerBuilder());
  const merged = mergeFaces(faces);
  for (const rectangle of merged.rectangles) {
    addInstance(builders[rectangle.quad.layer], rectangle.origin, rectangle.width, rectangle.height, rectangle.quad, rectangle.tint, [
      rectangle.light,
      rectangle.light,
      rectangle.light,
      rectangle.light,
    ]);
  }
  for (const face of merged.loose) {
    const builder = builders[face.quad.layer];
    if (isInstanceable(face.quad)) {
      addInstance(builder, [face.x, face.y, face.z], 1, 1, face.quad, face.tint, face.light);
    } else {
      addExplicit(builder, face, table);
    }
  }
  return builders.map((builder) => builder.finish());
}

function isInstanceable(quad: PreparedQuad): boolean {
  return quad.unit && quad.pattern >= 0;
}

function addInstance(
  builder: LayerBuilder,
  origin: number[],
  width: number,
  height: number,
  quad: PreparedQuad,
  tint: number[],
  light: number[],
): void {
  const [first, second] = TANGENTS[quad.direction];
  const templateLight = [0, 0, 0, 0];
  const corner = quad.corners;
  const templateIndex: number[] = [];
  for (let index = 0; index < 4; index++) {
    const s = corner[index * 3 + first] > 8 ? 1 : 0;
    const t = corner[index * 3 + second] > 8 ? 1 : 0;
    templateIndex.push(TEMPLATE_INDEX[s][t]);
    templateLight[TEMPLATE_INDEX[s][t]] = light[index];
  }
  const zeroTwo = light[0] + light[2] >= light[1] + light[3];
  const diagonal = zeroTwo ? [templateIndex[0], templateIndex[2]] : [templateIndex[1], templateIndex[3]];
  const splitsAlongZeroTwo = diagonal[0] + diagonal[1] === 2;

  builder.origin.push(origin[0], origin[1], origin[2]);
  builder.size.push(width, height);
  builder.direction.push(quad.direction);
  builder.shade.push(quad.shade);
  builder.pattern.push(quad.pattern | (splitsAlongZeroTwo ? 8 : 0));
  builder.cell.push(quad.cell[0], quad.cell[1]);
  builder.tint.push(tint[0], tint[1], tint[2]);
  builder.light.push(...templateLight);
  builder.count++;
}

/** Template corners run (0, 0), (1, 0), (1, 1), (0, 1) across the face's first and second tangents. */
const TEMPLATE_INDEX = [
  [0, 3],
  [1, 2],
];

function addExplicit(builder: LayerBuilder, face: Face, table: PreparedTable): void {
  const quad = face.quad;
  const base = builder.explicitQuads * 4;
  const uvs = insetUvs(quad, table);
  for (let corner = 0; corner < 4; corner++) {
    builder.positions.push(
      face.x + quad.corners[corner * 3] / 16,
      face.y + quad.corners[corner * 3 + 1] / 16,
      face.z + quad.corners[corner * 3 + 2] / 16,
    );
    builder.uvs.push(uvs[corner * 2], uvs[corner * 2 + 1]);
    const shade = FACE_SHADE[quad.shade];
    for (const channel of face.tint) {
      builder.colours.push((channel / 255) * shade * (face.light[corner] / 255));
    }
  }
  const light = face.light;
  const split = light[0] + light[2] >= light[1] + light[3] ? ZERO_TWO : ONE_THREE;
  for (const index of split) {
    builder.indices.push(base + index);
  }
  builder.explicitQuads++;
}

/** Atlas coordinates of the quad's corners, pulled inward by the half-texel inset along each axis, as the explicit path did. */
function insetUvs(quad: PreparedQuad, table: PreparedTable): number[] {
  const inset = [INSET_TEXELS / table.atlasWidth, INSET_TEXELS / table.atlasHeight];
  const values = Array.from(quad.uvs, (value) => value / 65535);
  for (let axis = 0; axis < 2; axis++) {
    const centre = (values[axis] + values[2 + axis] + values[4 + axis] + values[6 + axis]) / 4;
    for (let corner = 0; corner < 4; corner++) {
      const at = corner * 2 + axis;
      const distance = centre - values[at];
      if (Math.abs(distance) > inset[axis]) {
        values[at] += Math.sign(distance) * inset[axis];
      }
    }
  }
  return values;
}

class LayerBuilder {
  count = 0;
  origin: number[] = [];
  size: number[] = [];
  direction: number[] = [];
  shade: number[] = [];
  pattern: number[] = [];
  cell: number[] = [];
  tint: number[] = [];
  light: number[] = [];
  explicitQuads = 0;
  positions: number[] = [];
  uvs: number[] = [];
  colours: number[] = [];
  indices: number[] = [];

  finish(): LayerArrays {
    return {
      instances: {
        count: this.count,
        origin: Uint8Array.from(this.origin),
        size: Uint8Array.from(this.size),
        direction: Uint8Array.from(this.direction),
        shade: Uint8Array.from(this.shade),
        pattern: Uint8Array.from(this.pattern),
        cell: Uint16Array.from(this.cell),
        tint: Uint8Array.from(this.tint),
        light: Uint8Array.from(this.light),
      },
      explicit: {
        quads: this.explicitQuads,
        positions: Float32Array.from(this.positions),
        uvs: Float32Array.from(this.uvs),
        colours: Float32Array.from(this.colours),
        indices: Uint32Array.from(this.indices),
      },
    };
  }
}
