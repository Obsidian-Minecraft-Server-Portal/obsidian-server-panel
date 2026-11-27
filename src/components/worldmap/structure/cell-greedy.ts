// Greedy merging of a cell's faces into rectangles. Two faces merge only when they cover their block face exactly and share
// a texture layout, a tint and a constant brightness, so every block of a rectangle would have been drawn identically.

import { type Face, TANGENTS } from "./cell-mesh";
import { CELL_BLOCKS } from "./bundle";
import type { PreparedQuad } from "./tables";

export interface Rectangle {
  direction: number;
  quad: PreparedQuad;
  /** Block coordinates of the rectangle's first block, cell-local. */
  origin: [number, number, number];
  /** Blocks along the first and second tangent axes of the face. */
  width: number;
  height: number;
  tint: number[];
  /** The brightness shared by every corner of every block in the rectangle. */
  light: number;
}

export interface Merged {
  rectangles: Rectangle[];
  /** Faces that cannot merge, drawn one by one. */
  loose: Face[];
}

const GRID = CELL_BLOCKS * CELL_BLOCKS;

export function mergeFaces(faces: Face[]): Merged {
  const merged: Merged = { rectangles: [], loose: [] };
  for (let direction = 0; direction < 6; direction++) {
    const planes = new Map<number, (Face | undefined)[]>();
    const [first, second] = TANGENTS[direction];
    const normal = 3 - first - second;
    for (const face of faces) {
      if (face.quad.direction !== direction) {
        continue;
      }
      if (!isMergeable(face)) {
        merged.loose.push(face);
        continue;
      }
      const plane = coordinate(face, normal);
      const grid = planes.get(plane) ?? new Array<Face | undefined>(GRID);
      planes.set(plane, grid);
      const cell = coordinate(face, second) * CELL_BLOCKS + coordinate(face, first);
      if (grid[cell]) {
        merged.loose.push(face);
      } else {
        grid[cell] = face;
      }
    }
    for (const [plane, grid] of planes) {
      mergeGrid(grid, plane, normal, first, second, direction, merged);
    }
  }
  return merged;
}

function mergeGrid(
  grid: (Face | undefined)[],
  plane: number,
  normal: number,
  first: number,
  second: number,
  direction: number,
  merged: Merged,
): void {
  const used = new Uint8Array(GRID);
  const keys = grid.map((face) => (face ? keyOf(face) : ""));
  for (let v = 0; v < CELL_BLOCKS; v++) {
    for (let u = 0; u < CELL_BLOCKS; u++) {
      const start = v * CELL_BLOCKS + u;
      const face = grid[start];
      if (!face || used[start]) {
        continue;
      }
      const key = keys[start];
      let width = 1;
      while (u + width < CELL_BLOCKS && !used[start + width] && keys[start + width] === key && grid[start + width]) {
        width++;
      }
      let height = 1;
      grow: while (v + height < CELL_BLOCKS) {
        const row = (v + height) * CELL_BLOCKS + u;
        for (let column = 0; column < width; column++) {
          if (used[row + column] || keys[row + column] !== key || !grid[row + column]) {
            break grow;
          }
        }
        height++;
      }
      for (let row = 0; row < height; row++) {
        used.fill(1, start + row * CELL_BLOCKS, start + row * CELL_BLOCKS + width);
      }
      const origin: number[] = [0, 0, 0];
      origin[normal] = plane;
      origin[first] = u;
      origin[second] = v;
      merged.rectangles.push({
        direction,
        quad: face.quad,
        origin: origin as [number, number, number],
        width,
        height,
        tint: face.tint,
        light: face.light[0],
      });
    }
  }
}

function isMergeable(face: Face): boolean {
  const [a, b, c, d] = face.light;
  return face.quad.unit && face.quad.pattern >= 0 && a === b && b === c && c === d;
}

function coordinate(face: Face, axis: number): number {
  return axis === 0 ? face.x : axis === 1 ? face.y : face.z;
}

function keyOf(face: Face): string {
  return `${face.quad.layer}|${face.quad.pattern}|${face.quad.shade}|${face.quad.uvs.join(",")}|${face.tint.join(",")}|${face.light[0]}`;
}
