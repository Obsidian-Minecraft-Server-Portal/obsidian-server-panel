import { useEffect, useLayoutEffect, useRef, useState, type PointerEvent, type SyntheticEvent, type WheelEvent } from "react";
import {
  useWorldMapManifest,
  worldMapDetailUrl,
  worldMapSkinBase,
  worldMapSource,
  type LivePlayer,
  type WorldMapTile,
} from "../../api/worldmap";
import type { MapFocus } from "./map-focus";
import type { ViewPlace } from "./use-viewer-heartbeat";
import { dimensionMemory, rememberDimension } from "./map-memory";
import { PlayerTag } from "./PlayerTag";

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

/** A pannable, zoomable top-down view of every rendered region in a dimension. It keeps its centre and zoom per instance. */
export function TileView({
  instanceId,
  folder,
  dimensionId,
  players,
  revision,
  focus,
  onPan,
  onPlace,
}: {
  instanceId: number;
  folder: string;
  dimensionId: string;
  players: readonly LivePlayer[];
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
  const view = stored ?? defaultView(tiles);
  const appliedFocus = useRef(-1);
  const followed = focus ? players.find((player) => player.name === focus.name && player.dimension === dimensionId) : undefined;
  const followedX = followed?.position[0];
  const followedZ = followed?.position[2];

  useLayoutEffect(() => {
    if (!focus || followedX === undefined || followedZ === undefined) return;
    const fresh = focus.nonce !== appliedFocus.current;
    if (!fresh && !focus.follow) return;
    appliedFocus.current = focus.nonce;
    const zoomsIn = fresh && focus.zoom;
    setStored((current) => {
      const zoom = current?.zoom ?? DEFAULT_ZOOM;
      return { x: followedX, z: followedZ, zoom: zoomsIn ? Math.max(zoom, FOCUS_ZOOM) : zoom };
    });
  }, [focus, followedX, followedZ]);

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

  const origin = {
    x: tiles.length ? Math.min(...tiles.map((tile) => tile.x)) * REGION_BLOCKS : 0,
    z: tiles.length ? Math.min(...tiles.map((tile) => tile.z)) * REGION_BLOCKS : 0,
  };
  const source = worldMapSource(instanceId, folder);
  const skinBase = worldMapSkinBase(instanceId);
  const inDimension = players.filter((player) => player.dimension === dimensionId);
  const detail =
    manifest.data && view.zoom >= DETAIL_ZOOM ? detailTilesOnScreen(instanceId, folder, view, size, origin, tiles) : [];
  const transform = `translate(${size.width / 2 - (view.x - origin.x) * view.zoom}px, ${size.height / 2 - (view.z - origin.z) * view.zoom}px) scale(${view.zoom})`;

  return (
    <div
      ref={frame}
      className="absolute inset-0 cursor-grab touch-none select-none overflow-hidden active:cursor-grabbing"
      onPointerDown={(event: PointerEvent<HTMLDivElement>) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        onPan();
        setDrag({ pointerX: event.clientX, pointerY: event.clientY, view });
      }}
      onPointerMove={(event: PointerEvent<HTMLDivElement>) => {
        if (!drag) return;
        setStored({
          x: drag.view.x - (event.clientX - drag.pointerX) / drag.view.zoom,
          z: drag.view.z - (event.clientY - drag.pointerY) / drag.view.zoom,
          zoom: drag.view.zoom,
        });
      }}
      onPointerUp={() => setDrag(null)}
      onPointerCancel={() => setDrag(null)}
      onWheel={(event: WheelEvent<HTMLDivElement>) => {
        const rect = event.currentTarget.getBoundingClientRect();
        const offsetX = event.clientX - rect.left - size.width / 2;
        const offsetY = event.clientY - rect.top - size.height / 2;
        const zoom = clamp(view.zoom * (event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP), MIN_ZOOM, MAX_ZOOM);
        const pointX = view.x + offsetX / view.zoom;
        const pointZ = view.z + offsetY / view.zoom;
        setStored({ x: pointX - offsetX / zoom, z: pointZ - offsetY / zoom, zoom });
      }}
    >
      {manifest.isPending && <p className="p-4 text-sm text-ink-3">Loading tiles…</p>}
      {manifest.isError && <p className="p-4 text-sm text-rose">{manifest.error.message}</p>}
      {manifest.data && (
        <>
          <div className="absolute top-0 left-0 origin-top-left" style={{ transform }}>
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
              <PlayerTag
                key={player.name}
                player={player}
                skinBase={skinBase}
                x={size.width / 2 + (player.position[0] - view.x) * view.zoom}
                y={size.height / 2 + (player.position[2] - view.z) * view.zoom}
              />
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
