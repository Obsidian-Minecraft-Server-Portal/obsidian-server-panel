import {
	CanvasTexture,
	Group,
	MathUtils,
	MeshBasicMaterial,
	NearestFilter,
	PerspectiveCamera,
	Sprite,
	SpriteMaterial,
	SRGBColorSpace,
	TextureLoader,
	Vector3,
} from "three";
import type { LivePlayer } from "../../api/worldmap";
import type { PlayerPose } from "./player-track";
import { buildRig, CHARACTER_HEIGHT, disposeRig, type PlayerRig } from "./player-model";

/** Players are drawn 1.75 blocks tall at every distance, like a player in game. */
const PLAYER_HEIGHT = 1.75;
const RIG_SCALE = PLAYER_HEIGHT / CHARACTER_HEIGHT;
/** Name tags show only for players this close to the camera, in blocks. */
const TAG_RANGE = 30;
const TAG_PIXELS = 26;
const TAG_CANVAS_HEIGHT = 56;
const TAG_FONT_SIZE = 28;
const TAG_CHAR_WIDTH = TAG_FONT_SIZE * 0.6;
const TAG_PADDING = 14;
const TAG_FONT = `600 ${TAG_FONT_SIZE}px "JetBrains Mono", ui-monospace, monospace`;
const TAG_BACKGROUND = "rgba(10, 6, 9, 0.8)";
const TAG_LIFT = 0.25;
const GREY = 0x8a8a8a;
const RENDER_ORDER = 10;

interface Marker {
	player: LivePlayer;
	anchor: Vector3;
	rig: PlayerRig;
	skin: string;
	slim: boolean;
	legacy: boolean;
	tag: Sprite;
	tagTexture: CanvasTexture;
	tagAspect: number;
}

/** Player characters and name tags for the 3D view. Each character carries its skin, faces its yaw and tilts its head. */
export class PlayerMarkers {
	readonly group = new Group();
	private readonly skinBase: string;
	private readonly markers = new Map<string, Marker>();
	private readonly materials = new Map<string, MeshBasicMaterial>();
	private readonly legacySkins = new Set<string>();
	private readonly loader = new TextureLoader();
	private hidden: string | null = null;

	constructor(skinBase: string) {
		this.skinBase = skinBase;
	}

	update(players: readonly LivePlayer[]): void {
		const online = new Set<string>();
		for (const player of players) {
			online.add(player.name);
			const marker = this.markers.get(player.name) ?? this.add(player);
			marker.player = player;
			marker.anchor.set(player.position[0], player.position[1], player.position[2]);
			this.ensureRig(marker);
		}
		for (const name of [...this.markers.keys()]) {
			if (!online.has(name)) this.remove(name);
		}
	}

	frame(camera: PerspectiveCamera, viewportHeight: number, poseOf: (name: string) => PlayerPose | undefined): void {
		const worldPerPixel = (2 * Math.tan(MathUtils.degToRad(camera.fov) / 2)) / viewportHeight;
		for (const marker of this.markers.values()) {
			const { player, rig } = marker;
			const pose = poseOf(player.name) ?? { dimension: player.dimension, position: player.position, yaw: player.yaw, pitch: player.pitch };
			const hidden = player.name === this.hidden;
			marker.anchor.set(pose.position[0], pose.position[1], pose.position[2]);
			const distance = marker.anchor.distanceTo(camera.position);
			rig.root.visible = !hidden;
			rig.root.position.copy(marker.anchor);
			rig.root.rotation.set(0, MathUtils.degToRad(-pose.yaw), 0);
			rig.root.scale.setScalar(RIG_SCALE);
			rig.head.rotation.set(MathUtils.degToRad(pose.pitch), 0, 0);
			const tagHeight = TAG_PIXELS * distance * worldPerPixel;
			marker.tag.visible = !hidden && distance <= TAG_RANGE;
			marker.tag.position.set(marker.anchor.x, marker.anchor.y + PLAYER_HEIGHT + TAG_LIFT, marker.anchor.z);
			marker.tag.scale.set(tagHeight * marker.tagAspect, tagHeight, 1);
		}
	}

	hide(name: string | null): void {
		this.hidden = name;
	}

	dispose(): void {
		for (const name of [...this.markers.keys()]) this.remove(name);
		for (const material of this.materials.values()) {
			material.map?.dispose();
			material.dispose();
		}
		this.materials.clear();
	}

	private add(player: LivePlayer): Marker {
		const { texture, aspect } = nameTag(player.name);
		const tag = new Sprite(new SpriteMaterial({ map: texture, depthTest: false, transparent: true }));
		tag.center.set(0.5, 0);
		tag.renderOrder = RENDER_ORDER;
		const marker: Marker = {
			player,
			anchor: new Vector3(),
			rig: this.buildFor(player),
			skin: player.skin,
			slim: player.model === "slim",
			legacy: this.legacySkins.has(player.skin),
			tag,
			tagTexture: texture,
			tagAspect: aspect,
		};
		this.group.add(marker.rig.root, tag);
		this.markers.set(player.name, marker);
		return marker;
	}

	private remove(name: string): void {
		const marker = this.markers.get(name);
		if (!marker) return;
		disposeRig(marker.rig);
		this.group.remove(marker.tag);
		marker.tag.material.dispose();
		marker.tagTexture.dispose();
		this.markers.delete(name);
	}

	private ensureRig(marker: Marker): void {
		const { player } = marker;
		const slim = player.model === "slim";
		const legacy = this.legacySkins.has(player.skin);
		if (marker.skin === player.skin && marker.slim === slim && marker.legacy === legacy) return;
		disposeRig(marker.rig);
		marker.rig = this.buildFor(player);
		marker.skin = player.skin;
		marker.slim = slim;
		marker.legacy = legacy;
		this.group.add(marker.rig.root);
	}

	private buildFor(player: LivePlayer): PlayerRig {
		return buildRig(this.material(player.skin), player.model === "slim", this.legacySkins.has(player.skin));
	}

	/** The shared material of one skin. It is grey until the skin loads, and a legacy 64x32 skin rebuilds its characters. */
	private material(file: string): MeshBasicMaterial {
		const cached = this.materials.get(file);
		if (cached) return cached;
		const material = new MeshBasicMaterial({ vertexColors: true, alphaTest: 0.5, color: GREY });
		this.materials.set(file, material);
		this.loader.load(this.skinBase + file, (texture) => {
			texture.colorSpace = SRGBColorSpace;
			texture.magFilter = NearestFilter;
			texture.minFilter = NearestFilter;
			texture.generateMipmaps = false;
			material.map = texture;
			material.color.set(0xffffff);
			material.needsUpdate = true;
			if (texture.image.height === 32) {
				this.legacySkins.add(file);
				this.markers.forEach((marker) => this.ensureRig(marker));
			}
		});
		return material;
	}
}

function nameTag(name: string): { texture: CanvasTexture; aspect: number } {
	const canvas = document.createElement("canvas");
	canvas.width = Math.ceil(name.length * TAG_CHAR_WIDTH) + TAG_PADDING * 2;
	canvas.height = TAG_CANVAS_HEIGHT;
	const context = canvas.getContext("2d");
	if (context) {
		context.fillStyle = TAG_BACKGROUND;
		context.fillRect(0, 0, canvas.width, canvas.height);
		context.font = TAG_FONT;
		context.fillStyle = "#ffffff";
		context.textBaseline = "middle";
		context.fillText(name, TAG_PADDING, canvas.height / 2);
	}
	const texture = new CanvasTexture(canvas);
	texture.colorSpace = SRGBColorSpace;
	return { texture, aspect: canvas.width / canvas.height };
}
