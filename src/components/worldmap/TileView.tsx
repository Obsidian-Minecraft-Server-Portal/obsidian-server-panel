import { useEffect, useLayoutEffect, useRef, useState, type PointerEvent, type SyntheticEvent, type WheelEvent } from "react";
import {
  useWorldMapManifest,
  worldMapDetailUrl,
  worldMapSkinBase,
  worldMapSource,
  type LivePlayer,
  type WorldMapTile,
} from "../../api/worldmap";
import { easeFactor } from "./ease";
import type { MapFocus } from "./map-focus";
import { dimensionMemory, rememberDimension } from "./map-memory";
import { PlayerTag } from "./PlayerTag";
import { PoseDamper } from "./pose-damper";
import type { PlayerPose, PlayerTrack } from "./player-track";
import type { ViewPlace } from "./use-viewer-heartbeat";

/** One region's top-down overview is 512 pixels square, at one pixel per block. */
const REGION_BLOCKS = 512;
/** A detail tile covers this many blocks a side. The server draws it at eight pixels per block. */
const DETAIL_BLOCKS = 32;
const DETAIL_PER_REGION = REGION_BLOCKS / DETAIL_BLOCKS;
/** From this many screen pixels per block, textured detail tiles replace the flat overview. */
const DETAIL_ZOOM = 2;
/** Caps the tiles one view can request, so zooming in over a large world cannot flood the server. */
const MAX_DETAIL_TILES = 300;
const DEFAULT_ZOOM = 0.5;
const MIN_ZOOM = 0.05;
const MAX_ZOOM = 8;
const ZOOM_STEP = 1.15;
/** Looking at a player zooms in at least this far, close enough for the textured tiles. */
const FOCUS_ZOOM = 3;
/** How quickly the view glides to a new focus, how quickly it follows a player, and how quickly a head closes the gap to the
 * track, all per second. */
const GLIDE_RATE = 3;
const FOLLOW_RATE = 8;
const DRAW_RATE = 12;
/** While the view moves by itself, React is told of its place this often, in seconds. */
const COMMIT_EVERY = 0.25;
/** A glide ends once the centre is this close to its goal, in blocks, and the zoom this close to its goal. */
const SETTLED_BLOCKS = 0.05;
const SETTLED_ZOOM = 0.001;

/** The block at the centre of the view, and the zoom in screen pixels per block. */
interface View {
  x: number;
  z: number;
  zoom: number;
}

interface Drag {
  pointerX: number;
  pointerY: number;
  view: View;
}

interface Size {
  width: number;
  height: number;
}

interface DetailTile {
  key: string;
  src: string;
  left: number;
  top: number;
}

/** Where a new focus is gliding to: a centre, when the focus is not followed, and a zoom. A null field has arrived. */
interface Glide {
  centre: { x: number; z: number } | null;
  zoom: number | null;
}

/** A pannable, zoomable top-down view of every rendered region in a dimension. It keeps its centre and zoom per instance. The
 * view and the player heads move every frame between the places the track gives, so they never jump when a reading arrives. */
export function TileView({
  instanceId,
  folder,
  dimensionId,
  players,
  track,
  revision,
  focus,
  onPan,
  onPlace,
}: {
  instanceId: number;
  folder: string;
  dimensionId: string;
  players: readonly LivePlayer[];
  track: PlayerTrack;
  revision: number;
  focus: MapFocus | null;
  /** Called when the viewer drags the map, so following a player can stop. */
  onPan: () => void;
  onPlace: (place: ViewPlace) => void;
}) {
  const manifest = useWorldMapManifest(instanceId, folder, revision);
  const tiles = manifest.data?.tiles ?? [];
  const [stored, setStored] = useState<View | null>(() => storedView(instanceId, folder));
  const [drag, setDrag] = useState<Drag | null>(null);
  const [size, setSize] = useState<Size>({ width: 0, height: 0 });
  const frame = useRef<HTMLDivElement>(null);
  const layer = useRef<HTMLDivElement>(null);
  const markerElements = useRef(new Map<string, HTMLDivElement>());
  const view = stored ?? defaultView(tiles);
  const origin = {
    x: tiles.length ? Math.min(...tiles.map((tile) => tile.x)) * REGION_BLOCKS : 0,
    z: tiles.length ? Math.min(...tiles.map((tile) => tile.z)) * REGION_BLOCKS : 0,
  };
  const latest = useRef({ view, origin, size, players, dimensionId, track });
  latest.current = { view, origin, size, players, dimensionId, track };
  /** The view as it is drawn this frame, which can run ahead of the React state while the view glides or follows. */
  const shown = useRef<View | null>(null);
  const committed = useRef<View | null>(null);
  const following = useRef<{ name: string; damper: PoseDamper } | null>(null);
  const glide = useRef<Glide | null>(null);
  const appliedFocus = useRef(-1);

  useLayoutEffect(() => {
    if (!stored || stored === committed.current) return;
    committed.current = stored;
    shown.current = stored;
  }, [stored]);

  useLayoutEffect(() => {
    if (!focus) {
      following.current = null;
      glide.current = null;
      return;
    }
    if (focus.follow) {
      if (following.current?.name !== focus.name) following.current = { name: focus.name, damper: new PoseDamper() };
    } else {
      following.current = null;
    }
    if (focus.nonce === appliedFocus.current) return;
    appliedFocus.current = focus.nonce;
    const { track: current, dimensionId: dimension, view: fallback } = latest.current;
    const pose = current.pose(focus.name, performance.now());
    const here = pose && pose.dimension === dimension ? pose : undefined;
    const from = shown.current ?? fallback;
    glide.current = {
      centre: here && !focus.follow ? { x: here.position[0], z: here.position[2] } : null,
      zoom: focus.zoom ? Math.max(from.zoom, FOCUS_ZOOM) : null,
    };
  }, [focus]);

  useEffect(() => {
    if (stored) onPlace({ x: stored.x, y: 0, z: stored.z });
  }, [stored, onPlace]);

  useEffect(() => {
    if (!stored && tiles.length > 0) {
      setStored(defaultView(tiles));
    }
  }, [stored, tiles]);

  useEffect(() => {
    if (stored) {
      rememberDimension(instanceId, folder, { x: stored.x, z: stored.z, zoom: stored.zoom });
    }
  }, [instanceId, folder, stored]);

  useEffect(() => {
    const element = frame.current;
    if (!element) {
      return undefined;
    }
    const measure = () => setSize({ width: element.clientWidth, height: element.clientHeight });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    let raf = 0;
    let last = performance.now();
    let sinceCommit = 0;
    let moving = false;
    let disposed = false;
    const dampers = new Map<string, PoseDamper>();

    const commit = (next: View) => {
      committed.current = next;
      setStored(next);
    };

    const tick = (now: number) => {
      if (disposed) return;
      const seconds = Math.min((now - last) / 1000, 0.1);
      last = now;
      const state = latest.current;
      const next: View = { ...(shown.current ?? state.view) };
      let busy = false;

      const follow = following.current;
      if (follow) {
        const pose = state.track.pose(follow.name, now);
        const drawn = follow.damper.update(pose && pose.dimension === state.dimensionId ? pose : undefined, seconds, FOLLOW_RATE);
        if (drawn) {
          next.x = drawn.position[0];
          next.z = drawn.position[2];
          busy = true;
        }
      }
      const current = glide.current;
      if (current?.centre) {
        const goal = current.centre;
        const factor = easeFactor(GLIDE_RATE, seconds);
        next.x += (goal.x - next.x) * factor;
        next.z += (goal.z - next.z) * factor;
        if (Math.hypot(goal.x - next.x, goal.z - next.z) < SETTLED_BLOCKS) {
          next.x = goal.x;
          next.z = goal.z;
          current.centre = null;
        }
        busy = true;
      }
      if (current?.zoom != null) {
        const goal = current.zoom;
        next.zoom += (goal - next.zoom) * easeFactor(GLIDE_RATE, seconds);
        if (Math.abs(goal - next.zoom) < SETTLED_ZOOM) {
          next.zoom = goal;
          current.zoom = null;
        }
        busy = true;
      }
      if (current && current.centre === null && current.zoom === null) glide.current = null;
      shown.current = next;

      if (layer.current) {
        layer.current.style.transform = `translate(${state.size.width / 2 - (next.x - state.origin.x) * next.zoom}px, ${state.size.height / 2 - (next.z - state.origin.z) * next.zoom}px) scale(${next.zoom})`;
      }
      const names = new Set<string>();
      for (const player of state.players) {
        if (player.dimension !== state.dimensionId) continue;
        names.add(player.name);
        const element = markerElements.current.get(player.name);
        if (!element) continue;
        const damper = dampers.get(player.name) ?? new PoseDamper();
        dampers.set(player.name, damper);
        const pose = damper.update(state.track.pose(player.name, now) ?? fallbackPose(player), seconds, DRAW_RATE) ?? fallbackPose(player);
        const x = state.size.width / 2 + (pose.position[0] - next.x) * next.zoom;
        const y = state.size.height / 2 + (pose.position[2] - next.z) * next.zoom;
        element.style.transform = `translate(${x}px, ${y}px)`;
      }
      for (const name of [...dampers.keys()]) {
        if (!names.has(name)) dampers.delete(name);
      }

      if (busy) {
        sinceCommit += seconds;
        if (sinceCommit >= COMMIT_EVERY) {
          sinceCommit = 0;
          commit({ ...next });
        }
      } else if (moving) {
        sinceCommit = 0;
        commit({ ...next });
      }
      moving = busy;
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
    };
  }, []);

  const register = (name: string, element: HTMLDivElement | null) => {
    if (element) markerElements.current.set(name, element);
    else markerElements.current.delete(name);
  };
  const source = worldMapSource(instanceId, folder);
  const skinBase = worldMapSkinBase(instanceId);
  const inDimension = players.filter((player) => player.dimension === dimensionId);
  const detail = manifest.data && view.zoom >= DETAIL_ZOOM ? detailTilesOnScreen(instanceId, folder, view, size, origin, tiles) : [];

  return (
    <div
      ref={frame}
      className="absolute inset-0 cursor-grab touch-none select-none overflow-hidden active:cursor-grabbing"
      onPointerDown={(event: PointerEvent<HTMLDivElement>) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        onPan();
        glide.current = null;
        setDrag({ pointerX: event.clientX, pointerY: event.clientY, view: shown.current ?? view });
      }}
      onPointerMove={(event: PointerEvent<HTMLDivElement>) => {
        if (!drag) return;
        const next = {
          x: drag.view.x - (event.clientX - drag.pointerX) / drag.view.zoom,
          z: drag.view.z - (event.clientY - drag.pointerY) / drag.view.zoom,
          zoom: drag.view.zoom,
        };
        shown.current = next;
        committed.current = next;
        setStored(next);
      }}
      onPointerUp={() => setDrag(null)}
      onPointerCancel={() => setDrag(null)}
      onWheel={(event: WheelEvent<HTMLDivElement>) => {
        const rect = event.currentTarget.getBoundingClientRect();
        const current = shown.current ?? view;
        const offsetX = event.clientX - rect.left - size.width / 2;
        const offsetY = event.clientY - rect.top - size.height / 2;
        const zoom = clamp(current.zoom * (event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP), MIN_ZOOM, MAX_ZOOM);
        const pointX = current.x + offsetX / current.zoom;
        const pointZ = current.z + offsetY / current.zoom;
        glide.current = null;
        const next = { x: pointX - offsetX / zoom, z: pointZ - offsetY / zoom, zoom };
        shown.current = next;
        committed.current = next;
        setStored(next);
      }}
    >
      {manifest.isPending && <p className="p-4 text-sm text-ink-3">Loading tiles…</p>}
      {manifest.isError && <p className="p-4 text-sm text-rose">{manifest.error.message}</p>}
      {manifest.data && (
        <>
          <div ref={layer} className="absolute top-0 left-0 origin-top-left">
            {tiles.map((tile) => (
              <img
                key={tile.file}
                src={source.fileUrl(tile.file, tile.version)}
                alt=""
                loading="lazy"
                decoding="async"
                draggable={false}
                className="absolute max-w-none [image-rendering:pixelated]"
                style={{
                  left: tile.x * REGION_BLOCKS - origin.x,
                  top: tile.z * REGION_BLOCKS - origin.z,
                  width: REGION_BLOCKS,
                  height: REGION_BLOCKS,
                }}
              />
            ))}
            {detail.map((tile) => (
              <img
                key={tile.key}
                src={tile.src}
                alt=""
                decoding="async"
                draggable={false}
                onError={hideBroken}
                className="absolute max-w-none"
                style={{ left: tile.left, top: tile.top, width: DETAIL_BLOCKS, height: DETAIL_BLOCKS }}
              />
            ))}
          </div>
          <div className="pointer-events-none absolute inset-0">
            {inDimension.map((player) => (
              <div
                key={player.name}
                ref={(element) => register(player.name, element)}
                className="pointer-events-none absolute top-0 left-0"
              >
                <PlayerTag player={player} skinBase={skinBase} />
              </div>
            ))}
          </div>
          <div className="pointer-events-none absolute bottom-3 left-3 flex items-center gap-2 rounded-control border border-line bg-canvas/80 px-2.5 py-1.5 font-mono text-[11px] text-ink-2 shadow-lift backdrop-blur-md">
            <span>X {Math.round(view.x)}</span>
            <span>Z {Math.round(view.z)}</span>
            <span className="text-ink-3">{view.zoom >= 1 ? `${view.zoom.toFixed(1)}×` : `1:${Math.round(1 / view.zoom)}`}</span>
          </div>
        </>
      )}
    </div>
  );
}

/** The pose of a player as the player list gives it, for a head the track does not know yet. */
function fallbackPose(player: LivePlayer): PlayerPose {
  return { dimension: player.dimension, position: player.position, yaw: player.yaw, pitch: player.pitch };
}

/** The view a dimension was last left on, or `null` when it has never been centred. */
function storedView(instanceId: number, folder: string): View | null {
  const remembered = dimensionMemory(instanceId, folder);
  if (remembered.x === undefined || remembered.z === undefined) {
    return null;
  }
  return { x: remembered.x, z: remembered.z, zoom: clamp(remembered.zoom ?? DEFAULT_ZOOM, MIN_ZOOM, MAX_ZOOM) };
}

/** Centred on the middle of the rendered regions, so the first view shows all of them. */
function defaultView(tiles: WorldMapTile[]): View {
  if (tiles.length === 0) {
    return { x: 0, z: 0, zoom: DEFAULT_ZOOM };
  }
  const minX = Math.min(...tiles.map((tile) => tile.x));
  const maxX = Math.max(...tiles.map((tile) => tile.x));
  const minZ = Math.min(...tiles.map((tile) => tile.z));
  const maxZ = Math.max(...tiles.map((tile) => tile.z));
  return {
    x: ((minX + maxX + 1) * REGION_BLOCKS) / 2,
    z: ((minZ + maxZ + 1) * REGION_BLOCKS) / 2,
    zoom: DEFAULT_ZOOM,
  };
}

/** The detail tiles that cover the screen, for regions that have a render. Positions are in overview pixels. */
function detailTilesOnScreen(
  instanceId: number,
  folder: string,
  view: View,
  size: Size,
  origin: { x: number; z: number },
  tiles: WorldMapTile[],
): DetailTile[] {
  const regions = new Map(tiles.map((tile) => [`${tile.x},${tile.z}`, tile]));
  const halfWidth = size.width / 2 / view.zoom;
  const halfHeight = size.height / 2 / view.zoom;
  const firstX = Math.floor((view.x - halfWidth) / DETAIL_BLOCKS);
  const lastX = Math.floor((view.x + halfWidth) / DETAIL_BLOCKS);
  const firstZ = Math.floor((view.z - halfHeight) / DETAIL_BLOCKS);
  const lastZ = Math.floor((view.z + halfHeight) / DETAIL_BLOCKS);
  const found: DetailTile[] = [];
  for (let tileX = firstX; tileX <= lastX; tileX++) {
    for (let tileZ = firstZ; tileZ <= lastZ; tileZ++) {
      const regionX = Math.floor(tileX / DETAIL_PER_REGION);
      const regionZ = Math.floor(tileZ / DETAIL_PER_REGION);
      const region = regions.get(`${regionX},${regionZ}`);
      if (!region) continue;
      found.push({
        key: `${tileX},${tileZ}`,
        src: worldMapDetailUrl(
          instanceId,
          folder,
          regionX,
          regionZ,
          tileX - regionX * DETAIL_PER_REGION,
          tileZ - regionZ * DETAIL_PER_REGION,
          region.version,
        ),
        left: tileX * DETAIL_BLOCKS - origin.x,
        top: tileZ * DETAIL_BLOCKS - origin.z,
      });
      if (found.length >= MAX_DETAIL_TILES) return found;
    }
  }
  return found;
}

/** A tile the server could not draw keeps the overview underneath it. */
function hideBroken(event: SyntheticEvent<HTMLImageElement>) {
  event.currentTarget.style.visibility = "hidden";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
