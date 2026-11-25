// Turns a cell's arrays into three.js objects: an instanced mesh and an explicit mesh per render layer, placed at the cell's corner.

import { BufferAttribute, BufferGeometry, Group, Mesh, MeshBasicMaterial, type ShaderMaterial, type Texture, Vector2 } from "three";
import { CELL_BLOCKS } from "./bundle";
import { createInstanceGeometry, createInstanceMaterial } from "./cell-shader";
import type { LayerArrays } from "./instances";

export interface CellMaterials {
  /** Per layer: opaque, cutout, translucent. */
  instanced: ShaderMaterial[];
  explicit: MeshBasicMaterial[];
}

export function createCellMaterials(atlas: Texture, atlasWidth: number, atlasHeight: number): CellMaterials {
  const atlasSize = new Vector2(atlasWidth, atlasHeight);
  return {
    instanced: [
      createInstanceMaterial({ atlas, atlasSize, cutout: false, translucent: false }),
      createInstanceMaterial({ atlas, atlasSize, cutout: true, translucent: false }),
      createInstanceMaterial({ atlas, atlasSize, cutout: false, translucent: true }),
    ],
    explicit: [
      new MeshBasicMaterial({ map: atlas, vertexColors: true }),
      new MeshBasicMaterial({ map: atlas, vertexColors: true, alphaTest: 0.5 }),
      new MeshBasicMaterial({ map: atlas, vertexColors: true, transparent: true, depthWrite: false }),
    ],
  };
}

export interface CellObject {
  group: Group;
  dispose(): void;
}

export function buildCell(cell: [number, number, number], layers: LayerArrays[], materials: CellMaterials): CellObject {
  const group = new Group();
  group.position.set(cell[0] * CELL_BLOCKS, cell[1] * CELL_BLOCKS, cell[2] * CELL_BLOCKS);
  const geometries: BufferGeometry[] = [];
  layers.forEach((layer, index) => {
    if (layer.instances.count > 0) {
      const geometry = createInstanceGeometry(layer.instances);
      geometries.push(geometry);
      group.add(new Mesh(geometry, materials.instanced[index]));
    }
    if (layer.explicit.quads > 0) {
      const geometry = new BufferGeometry();
      geometry.setAttribute("position", new BufferAttribute(layer.explicit.positions, 3));
      geometry.setAttribute("uv", new BufferAttribute(layer.explicit.uvs, 2));
      geometry.setAttribute("color", new BufferAttribute(layer.explicit.colours, 3));
      geometry.setIndex(new BufferAttribute(layer.explicit.indices, 1));
      geometry.computeBoundingSphere();
      geometries.push(geometry);
      group.add(new Mesh(geometry, materials.explicit[index]));
    }
  });
  return {
    group,
    dispose: () => geometries.forEach((geometry) => geometry.dispose()),
  };
}
