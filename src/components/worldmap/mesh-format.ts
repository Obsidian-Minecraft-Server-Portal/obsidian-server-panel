// Decodes OBSM version 2 section meshes written by obsidian-worldmap (crates/obsidian-worldmap/src/mesh/format.rs).

const MAGIC = "OBSM";
const VERSION = 2;
const HEADER_LENGTH = 32;
const QUAD_LENGTH = 48;
const CORNER_LENGTH = 10;
const LIGHT_OFFSET = 44;

/** Vanilla's directional brightness, in the order down, up, north, south, west, east. */
const FACE_SHADE = [0.5, 1, 0.8, 0.8, 0.6, 0.6];
/** Triangles that split a quad along its 0-2 diagonal, or along its 1-3 diagonal. */
const SPLIT_ALONG_ZERO_TWO = [0, 1, 2, 0, 2, 3];
const SPLIT_ALONG_ONE_THREE = [0, 1, 3, 1, 2, 3];

export interface LayerMesh {
  /** Block units relative to the section origin, four corners per quad. */
  positions: Float32Array;
  /** Atlas coordinates in image space (origin at the top left), 0..1. */
  uvs: Float32Array;
  /** Tint, face shade and corner light multiplied together, RGB 0..1. */
  colours: Float32Array;
  indices: Uint32Array;
  quadCount: number;
}

export interface SectionMesh {
  /** Chunk x, section y and chunk z. */
  section: [number, number, number];
  /** Opaque, cutout and translucent layers, in that order. */
  layers: [LayerMesh, LayerMesh, LayerMesh];
}

export function decodeSectionMesh(buffer: ArrayBuffer): SectionMesh {
  if (buffer.byteLength < HEADER_LENGTH) {
    throw new Error("section mesh is shorter than its header");
  }
  const view = new DataView(buffer);
  if (String.fromCharCode(...new Uint8Array(buffer, 0, 4)) !== MAGIC) {
    throw new Error("section mesh has a bad magic");
  }
  const version = view.getUint16(4, true);
  if (version !== VERSION) {
    throw new Error(`unsupported section mesh version ${version}`);
  }
  const section: [number, number, number] = [view.getInt32(8, true), view.getInt32(12, true), view.getInt32(16, true)];
  const counts = [0, 1, 2].map((layer) => view.getUint32(20 + layer * 4, true));
  let offset = HEADER_LENGTH;
  const layers = counts.map((count) => {
    const layer = decodeLayer(view, offset, count);
    offset += count * QUAD_LENGTH;
    return layer;
  });
  if (offset > buffer.byteLength) {
    throw new Error("section mesh is truncated");
  }
  return { section, layers: layers as SectionMesh["layers"] };
}

function decodeLayer(view: DataView, start: number, count: number): LayerMesh {
  const positions = new Float32Array(count * 12);
  const uvs = new Float32Array(count * 8);
  const colours = new Float32Array(count * 12);
  const indices = new Uint32Array(count * 6);
  const light = [0, 0, 0, 0];
  for (let quad = 0; quad < count; quad++) {
    const base = start + quad * QUAD_LENGTH;
    const shade = FACE_SHADE[view.getUint8(base + 40)] ?? 1;
    const tint = [view.getUint8(base + 41), view.getUint8(base + 42), view.getUint8(base + 43)];
    for (let corner = 0; corner < 4; corner++) {
      const at = base + corner * CORNER_LENGTH;
      const vertex = quad * 4 + corner;
      light[corner] = view.getUint8(base + LIGHT_OFFSET + corner);
      for (let axis = 0; axis < 3; axis++) {
        positions[vertex * 3 + axis] = view.getInt16(at + axis * 2, true) / 256;
        colours[vertex * 3 + axis] = (tint[axis] / 255) * shade * (light[corner] / 255);
      }
      uvs[vertex * 2] = view.getUint16(at + 6, true) / 65535;
      uvs[vertex * 2 + 1] = view.getUint16(at + 8, true) / 65535;
    }
    const split = light[0] + light[2] >= light[1] + light[3] ? SPLIT_ALONG_ZERO_TWO : SPLIT_ALONG_ONE_THREE;
    const first = quad * 4;
    for (let index = 0; index < 6; index++) {
      indices[quad * 6 + index] = first + split[index];
    }
  }
  return { positions, uvs, colours, indices, quadCount: count };
}
