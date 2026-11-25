import { BoxGeometry, Float32BufferAttribute, Group, Mesh, MeshBasicMaterial } from "three";

/** A player model is 32 pixels tall and the game draws it at 15/16 scale, which makes a player 1.875 blocks tall. */
export const CHARACTER_HEIGHT = 1.875;

const PIXEL = 1 / 16;
const SKIN_SIZE = 64;

/** A box of the skin texture: where its unfolded faces start, and its size in pixels. */
export interface SkinBox {
	u: number;
	v: number;
	w: number;
	h: number;
	d: number;
}

interface Face {
	normal: [number, number, number];
	/** The direction that runs left to right across the face, seen from outside. */
	right: [number, number, number];
	/** The direction that runs bottom to top, seen from outside. */
	up: [number, number, number];
	/** Face brightness, the way the game shades each side. */
	shade: number;
	/** The texel rectangle of this face on the unfolded box: left, top, right, bottom. */
	texels: (box: SkinBox) => [number, number, number, number];
}

const FACES: Face[] = [
	{
		normal: [0, 0, 1],
		right: [1, 0, 0],
		up: [0, 1, 0],
		shade: 0.8,
		texels: (b) => [b.u + b.d, b.v + b.d, b.u + b.d + b.w, b.v + b.d + b.h],
	},
	{
		normal: [0, 0, -1],
		right: [-1, 0, 0],
		up: [0, 1, 0],
		shade: 0.8,
		texels: (b) => [b.u + 2 * b.d + b.w, b.v + b.d, b.u + 2 * b.d + 2 * b.w, b.v + b.d + b.h],
	},
	{
		normal: [-1, 0, 0],
		right: [0, 0, 1],
		up: [0, 1, 0],
		shade: 0.6,
		texels: (b) => [b.u, b.v + b.d, b.u + b.d, b.v + b.d + b.h],
	},
	{
		normal: [1, 0, 0],
		right: [0, 0, -1],
		up: [0, 1, 0],
		shade: 0.6,
		texels: (b) => [b.u + b.d + b.w, b.v + b.d, b.u + 2 * b.d + b.w, b.v + b.d + b.h],
	},
	{
		normal: [0, 1, 0],
		right: [1, 0, 0],
		up: [0, 0, -1],
		shade: 1,
		texels: (b) => [b.u + b.d, b.v, b.u + b.d + b.w, b.v + b.d],
	},
	{
		normal: [0, -1, 0],
		right: [1, 0, 0],
		up: [0, 0, 1],
		shade: 0.5,
		texels: (b) => [b.u + b.d + b.w, b.v, b.u + b.d + 2 * b.w, b.v + b.d],
	},
];

const dot = (a: readonly number[], b: readonly number[]) => a.reduce((sum, value, index) => sum + value * b[index], 0);
const abs = (vector: readonly number[]) => vector.map(Math.abs);

/**
 * Lays a box's faces onto its part of the skin, as the game does: each face shows its own texels, upright when seen from
 * outside. A mirrored box reads the texture from the other side, which is how 64x32 skins draw their left limbs.
 */
function unfold(geometry: BoxGeometry, box: SkinBox, mirror: boolean): void {
	const { width, height, depth } = geometry.parameters;
	const half = [width / 2, height / 2, depth / 2];
	const { position, normal } = geometry.attributes;
	const uv = new Float32Array(position.count * 2);
	const color = new Float32Array(position.count * 3);
	for (let index = 0; index < position.count; index += 1) {
		const direction = [normal.getX(index), normal.getY(index), normal.getZ(index)].map(Math.round);
		const face = FACES.find((candidate) => candidate.normal.every((component, axis) => component === direction[axis]));
		if (!face) continue;
		const point = [position.getX(index), position.getY(index), position.getZ(index)];
		const s = dot(point, face.right) / dot(half, abs(face.right));
		const t = dot(point, face.up) / dot(half, abs(face.up));
		const [left, top, right, bottom] = face.texels(box);
		const across = mirror ? (1 - s) / 2 : (s + 1) / 2;
		const down = (1 - t) / 2;
		uv[index * 2] = (left + across * (right - left)) / SKIN_SIZE;
		uv[index * 2 + 1] = 1 - (top + down * (bottom - top)) / SKIN_SIZE;
		color.set([face.shade, face.shade, face.shade], index * 3);
	}
	geometry.setAttribute("uv", new Float32BufferAttribute(uv, 2));
	geometry.setAttribute("color", new Float32BufferAttribute(color, 3));
}

interface Limb {
	/** Where the limb hinges, in pixels from the feet. */
	joint: [number, number, number];
	/** Where the limb's box sits relative to its hinge, in pixels. */
	centre: [number, number, number];
	size: { w: number; h: number; d: number };
	base: { u: number; v: number };
	overlay?: { u: number; v: number; inflate: number };
	mirror?: boolean;
}

/** A character rig: a root at the feet, and a head pivot the pitch turns. */
export interface PlayerRig {
	root: Group;
	head: Group;
	meshes: Mesh[];
}

/**
 * Builds a player from boxes that carry the skin. A wide model has four-pixel arms and a slim model has three-pixel arms.
 * Legacy 64x32 skins have no second layer, and their left limbs reuse the right limbs mirrored.
 */
export function buildRig(material: MeshBasicMaterial, slim: boolean, legacy: boolean): PlayerRig {
	const arm = slim ? 3 : 4;
	const armX = 4 + arm / 2;
	const limbs: Limb[] = [
		{ joint: [0, 24, 0], centre: [0, 4, 0], size: { w: 8, h: 8, d: 8 }, base: { u: 0, v: 0 }, overlay: { u: 32, v: 0, inflate: 0.5 } },
		{ joint: [0, 0, 0], centre: [0, 18, 0], size: { w: 8, h: 12, d: 4 }, base: { u: 16, v: 16 }, overlay: { u: 16, v: 32, inflate: 0.25 } },
		{ joint: [-armX, 24, 0], centre: [0, -6, 0], size: { w: arm, h: 12, d: 4 }, base: { u: 40, v: 16 }, overlay: { u: 40, v: 32, inflate: 0.25 } },
		{ joint: [armX, 24, 0], centre: [0, -6, 0], size: { w: arm, h: 12, d: 4 }, base: legacy ? { u: 40, v: 16 } : { u: 32, v: 48 }, overlay: legacy ? undefined : { u: 48, v: 48, inflate: 0.25 }, mirror: legacy },
		{ joint: [-2, 12, 0], centre: [0, -6, 0], size: { w: 4, h: 12, d: 4 }, base: { u: 0, v: 16 }, overlay: { u: 0, v: 32, inflate: 0.25 } },
		{ joint: [2, 12, 0], centre: [0, -6, 0], size: { w: 4, h: 12, d: 4 }, base: legacy ? { u: 0, v: 16 } : { u: 16, v: 48 }, overlay: legacy ? undefined : { u: 0, v: 48, inflate: 0.25 }, mirror: legacy },
	];
	const root = new Group();
	const meshes: Mesh[] = [];
	const hinges = limbs.map((limb) => {
		const hinge = new Group();
		hinge.position.set(limb.joint[0] * PIXEL, limb.joint[1] * PIXEL, limb.joint[2] * PIXEL);
		const add = (box: SkinBox, inflate: number, mirror: boolean) => {
			const geometry = new BoxGeometry((box.w + 2 * inflate) * PIXEL, (box.h + 2 * inflate) * PIXEL, (box.d + 2 * inflate) * PIXEL);
			unfold(geometry, box, mirror);
			const mesh = new Mesh(geometry, material);
			mesh.position.set(limb.centre[0] * PIXEL, limb.centre[1] * PIXEL, limb.centre[2] * PIXEL);
			hinge.add(mesh);
			meshes.push(mesh);
		};
		add({ ...limb.base, ...limb.size }, 0, limb.mirror ?? false);
		if (limb.overlay) add({ u: limb.overlay.u, v: limb.overlay.v, ...limb.size }, limb.overlay.inflate, false);
		root.add(hinge);
		return hinge;
	});
	return { root, head: hinges[0], meshes };
}

/** Frees the geometry of a rig built by `buildRig`. Its material is shared and stays with the caller. */
export function disposeRig(rig: PlayerRig): void {
	rig.root.removeFromParent();
	rig.meshes.forEach((mesh) => mesh.geometry.dispose());
}
