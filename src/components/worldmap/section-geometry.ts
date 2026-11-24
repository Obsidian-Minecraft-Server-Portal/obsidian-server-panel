import { BufferAttribute, BufferGeometry } from "three";
import type { LayerMesh } from "./mesh-format";

/** Geometry for one render layer of a section. Positions are section-relative, in block units. */
export function buildLayerGeometry(layer: LayerMesh): BufferGeometry {
  const geometry = new BufferGeometry();
  geometry.setAttribute("position", new BufferAttribute(layer.positions, 3));
  geometry.setAttribute("uv", new BufferAttribute(layer.uvs, 2));
  geometry.setAttribute("color", new BufferAttribute(layer.colours, 3));
  geometry.setIndex(new BufferAttribute(layer.indices, 1));
  return geometry;
}
