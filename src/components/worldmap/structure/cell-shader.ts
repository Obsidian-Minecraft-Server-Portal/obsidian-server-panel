// The instanced shader that draws a cell's unit faces. Each instance is one face or merged rectangle of blocks; the shader
// works out its corners, its texture coordinates and its brightness from the instance fields the mesher writes.

import {
  DoubleSide,
  Float32BufferAttribute,
  GLSL3,
  InstancedBufferAttribute,
  InstancedBufferGeometry,
  ShaderMaterial,
  Sphere,
  Texture,
  Vector2,
  Vector3,
} from "three";
import type { InstanceArrays } from "./instances";

const VERTEX = /* glsl */ `in vec3 aOrigin;
in vec2 aSize;
in float aDirection;
in float aPattern;
in vec2 aCell;
in vec3 aTint;
in vec4 aLight;
out vec2 vAlong;
out vec2 vFace;
flat out float vDirection;
flat out float vPattern;
flat out float vUseA;
flat out vec2 vCell;
flat out vec3 vTint;
flat out vec4 vLight;
flat out float vFront;

const int FIRST_AXIS[6] = int[6](0, 0, 0, 0, 1, 1);
const int SECOND_AXIS[6] = int[6](2, 2, 1, 1, 2, 2);
const int NORMAL_AXIS[6] = int[6](1, 1, 2, 2, 0, 0);
const float PLANE_OFFSET[6] = float[6](0.0, 1.0, 0.0, 1.0, 0.0, 1.0);

vec3 axis(int index) {
  return index == 0 ? vec3(1.0, 0.0, 0.0) : (index == 1 ? vec3(0.0, 1.0, 0.0) : vec3(0.0, 0.0, 1.0));
}

void main() {
  int d = int(aDirection + 0.5);
  vec3 corner = aOrigin
    + axis(FIRST_AXIS[d]) * (position.x * aSize.x)
    + axis(SECOND_AXIS[d]) * (position.y * aSize.y)
    + axis(NORMAL_AXIS[d]) * PLANE_OFFSET[d];
  vFace = position.xy;
  vAlong = position.xy * aSize;
  vDirection = aDirection;
  vUseA = step(8.0, aPattern);
  vPattern = aPattern - vUseA * 8.0;
  vCell = aCell;
  vTint = aTint / 255.0;
  vLight = aLight / 255.0;
  vFront = (d == 0 || d == 3 || d == 5) ? 1.0 : 0.0;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(corner, 1.0);
}
`;

const FRAGMENT = /* glsl */ `in vec2 vAlong;
in vec2 vFace;
flat in float vDirection;
flat in float vPattern;
flat in float vUseA;
flat in vec2 vCell;
flat in vec3 vTint;
flat in vec4 vLight;
flat in float vFront;
uniform sampler2D uAtlas;
uniform vec2 uAtlasSize;
uniform float uCutout;
out vec4 outColour;

const float SHADE[6] = float[6](0.5, 1.0, 0.8, 0.8, 0.6, 0.6);

vec2 symmetry(int p, vec2 q) {
  if (p == 0) return vec2(q.x, q.y);
  if (p == 1) return vec2(1.0 - q.x, q.y);
  if (p == 2) return vec2(q.x, 1.0 - q.y);
  if (p == 3) return vec2(1.0 - q.x, 1.0 - q.y);
  if (p == 4) return vec2(q.y, q.x);
  if (p == 5) return vec2(1.0 - q.y, q.x);
  if (p == 6) return vec2(q.y, 1.0 - q.x);
  return vec2(1.0 - q.y, 1.0 - q.x);
}

float lightAt(vec2 st) {
  vec4 L = vLight;
  if (vUseA > 0.5) {
    if (st.x >= st.y) return (1.0 - st.x) * L.x + (st.x - st.y) * L.y + st.y * L.z;
    return (1.0 - st.y) * L.x + st.x * L.z + (st.y - st.x) * L.w;
  }
  if (st.x + st.y <= 1.0) return (1.0 - st.x - st.y) * L.x + st.x * L.y + st.y * L.w;
  return (1.0 - st.y) * L.y + (st.x + st.y - 1.0) * L.z + (1.0 - st.x) * L.w;
}

vec3 toSRGB(vec3 c) {
  return mix(12.92 * c, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(0.0031308, c));
}

void main() {
  if ((gl_FrontFacing ? 1.0 : 0.0) != vFront) discard;
  vec2 local = vAlong - floor(vAlong - 1e-4);
  vec2 texel = vec2(0.5) + 15.0 * symmetry(int(vPattern + 0.5), local);
  vec4 sampled = texture(uAtlas, (vCell + texel) / uAtlasSize);
  if (uCutout > 0.5 && sampled.a < 0.5) discard;
  vec3 colour = sampled.rgb * vTint * SHADE[int(vDirection + 0.5)] * lightAt(vFace);
  outColour = vec4(toSRGB(colour), sampled.a);
}
`;

const TEMPLATE_CORNERS = new Float32Array([0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0]);
const TEMPLATE_TRIANGLES = [0, 1, 2, 0, 2, 3];
/** Covers the whole block, and so every face and rectangle a cell can hold. */
const CELL_CENTRE = new Vector3(32, 32, 32);
const CELL_RADIUS = 56;

export interface InstanceMaterialOptions {
  atlas: Texture;
  atlasSize: Vector2;
  cutout: boolean;
  translucent: boolean;
}

export function createInstanceMaterial({ atlas, atlasSize, cutout, translucent }: InstanceMaterialOptions): ShaderMaterial {
  return new ShaderMaterial({
    glslVersion: GLSL3,
    vertexShader: VERTEX,
    fragmentShader: FRAGMENT,
    uniforms: {
      uAtlas: { value: atlas },
      uAtlasSize: { value: atlasSize },
      uCutout: { value: cutout ? 1 : 0 },
    },
    side: DoubleSide,
    transparent: translucent,
    depthWrite: !translucent,
  });
}

export function createInstanceGeometry(instances: InstanceArrays): InstancedBufferGeometry {
  const geometry = new InstancedBufferGeometry();
  geometry.setAttribute("position", new Float32BufferAttribute(TEMPLATE_CORNERS, 3));
  geometry.setIndex(TEMPLATE_TRIANGLES);
  geometry.setAttribute("aOrigin", new InstancedBufferAttribute(instances.origin, 3));
  geometry.setAttribute("aSize", new InstancedBufferAttribute(instances.size, 2));
  geometry.setAttribute("aDirection", new InstancedBufferAttribute(instances.direction, 1));
  geometry.setAttribute("aPattern", new InstancedBufferAttribute(instances.pattern, 1));
  geometry.setAttribute("aCell", new InstancedBufferAttribute(instances.cell, 2));
  geometry.setAttribute("aTint", new InstancedBufferAttribute(instances.tint, 3));
  geometry.setAttribute("aLight", new InstancedBufferAttribute(instances.light, 4));
  geometry.instanceCount = instances.count;
  geometry.boundingSphere = new Sphere(CELL_CENTRE, CELL_RADIUS);
  return geometry;
}
