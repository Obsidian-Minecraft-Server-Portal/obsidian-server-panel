import {
  CanvasTexture,
  Group,
  MathUtils,
  NearestFilter,
  PerspectiveCamera,
  Sprite,
  SpriteMaterial,
  SRGBColorSpace,
  Texture,
  TextureLoader,
  Vector3,
} from "three";
import type { LivePlayer } from "../../api/worldmap";

const HEAD_PIXELS = 28;
const TAG_PIXELS = 28;
const TAG_CANVAS_HEIGHT = 56;
const TAG_FONT_SIZE = 28;
const TAG_CHAR_WIDTH = TAG_FONT_SIZE * 0.6;
const TAG_PADDING = 14;
const TAG_FONT = `600 ${TAG_FONT_SIZE}px "JetBrains Mono", ui-monospace, monospace`;
const TAG_BACKGROUND = "rgba(10, 6, 9, 0.8)";
const HEAD_HEIGHT = 2;
const RENDER_ORDER = 10;

interface Marker {
  anchor: Vector3;
  file: string;
  head: Sprite;
  tag: Sprite;
  tagTexture: CanvasTexture;
  tagAspect: number;
}

/** Player heads and name tags for the 3D view. Each keeps the same size on screen whatever its distance. */
export class PlayerMarkers {
  readonly group = new Group();
  private readonly headBase: string;
  private readonly markers = new Map<string, Marker>();
  private readonly heads = new Map<string, Texture>();
  private readonly loader = new TextureLoader();

  constructor(headBase: string) {
    this.headBase = headBase;
  }

  update(players: readonly LivePlayer[]): void {
    const online = new Set<string>();
    for (const player of players) {
      online.add(player.name);
      const marker = this.markers.get(player.name) ?? this.add(player);
      marker.anchor.set(player.position[0], player.position[1] + HEAD_HEIGHT, player.position[2]);
      if (marker.file !== player.head) {
        marker.file = player.head;
        marker.head.material.map = this.headTexture(player.head);
        marker.head.material.needsUpdate = true;
      }
    }
    for (const name of [...this.markers.keys()]) {
      if (!online.has(name)) this.remove(name);
    }
  }

  frame(camera: PerspectiveCamera, viewportHeight: number): void {
    const worldPerPixel = (2 * Math.tan(MathUtils.degToRad(camera.fov) / 2)) / viewportHeight;
    for (const marker of this.markers.values()) {
      const scale = marker.anchor.distanceTo(camera.position) * worldPerPixel;
      const headSize = HEAD_PIXELS * scale;
      marker.head.visible = Boolean(marker.head.material.map?.image);
      marker.head.position.copy(marker.anchor);
      marker.head.scale.setScalar(headSize);
      marker.tag.position.set(marker.anchor.x, marker.anchor.y + headSize / 2, marker.anchor.z);
      marker.tag.scale.set(TAG_PIXELS * marker.tagAspect * scale, TAG_PIXELS * scale, 1);
    }
  }

  dispose(): void {
    for (const name of [...this.markers.keys()]) this.remove(name);
    for (const texture of this.heads.values()) texture.dispose();
    this.heads.clear();
  }

  private add(player: LivePlayer): Marker {
    const head = new Sprite(new SpriteMaterial({ map: this.headTexture(player.head), depthTest: false, transparent: true }));
    const { texture, aspect } = nameTag(player.name);
    const tag = new Sprite(new SpriteMaterial({ map: texture, depthTest: false, transparent: true }));
    tag.center.set(0.5, 0);
    head.renderOrder = RENDER_ORDER;
    tag.renderOrder = RENDER_ORDER;
    this.group.add(head, tag);
    const marker: Marker = { anchor: new Vector3(), file: player.head, head, tag, tagTexture: texture, tagAspect: aspect };
    this.markers.set(player.name, marker);
    return marker;
  }

  private remove(name: string): void {
    const marker = this.markers.get(name);
    if (!marker) return;
    this.group.remove(marker.head, marker.tag);
    marker.head.material.dispose();
    marker.tag.material.dispose();
    marker.tagTexture.dispose();
    this.markers.delete(name);
  }

  private headTexture(file: string): Texture {
    let texture = this.heads.get(file);
    if (!texture) {
      texture = this.loader.load(this.headBase + file);
      texture.colorSpace = SRGBColorSpace;
      texture.magFilter = NearestFilter;
      texture.minFilter = NearestFilter;
      texture.generateMipmaps = false;
      this.heads.set(file, texture);
    }
    return texture;
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
