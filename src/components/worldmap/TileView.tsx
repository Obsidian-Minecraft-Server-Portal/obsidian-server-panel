import { useState, type PointerEvent, type WheelEvent } from "react";
import { useWorldMapManifest, worldMapHeadBase, worldMapSource, type LivePlayer } from "../../api/worldmap";
import { PlayerTag } from "./PlayerTag";

/** One region's top-down tile is 512 pixels square, at one pixel per block. */
const TILE_PIXELS = 512;
const MIN_ZOOM = 0.05;
const MAX_ZOOM = 4;

interface View {
  x: number;
  y: number;
  zoom: number;
}

interface Drag {
  pointerX: number;
  pointerY: number;
  x: number;
  y: number;
}

/** A pannable, zoomable top-down view of every rendered region in a dimension. */
export function TileView({
  instanceId,
  folder,
  dimensionId,
  players,
}: {
  instanceId: number;
  folder: string;
  dimensionId: string;
  players: readonly LivePlayer[];
}) {
  const manifest = useWorldMapManifest(instanceId, folder);
  const [view, setView] = useState<View>({ x: 40, y: 40, zoom: 0.5 });
  const [drag, setDrag] = useState<Drag | null>(null);

  if (manifest.isPending) return <p className="p-4 text-sm text-ink-3">Loading tiles…</p>;
  if (manifest.isError) return <p className="p-4 text-sm text-rose">{manifest.error.message}</p>;

  const { tiles } = manifest.data;
  const originX = Math.min(...tiles.map((tile) => tile.x));
  const originZ = Math.min(...tiles.map((tile) => tile.z));
  const source = worldMapSource(instanceId, folder);
  const headBase = worldMapHeadBase(instanceId);
  const inDimension = players.filter((player) => player.dimension === dimensionId);

  return (
    <div
      className="relative h-full min-h-[60vh] w-full cursor-grab touch-none select-none overflow-hidden active:cursor-grabbing"
      onPointerDown={(event: PointerEvent<HTMLDivElement>) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        setDrag({ pointerX: event.clientX, pointerY: event.clientY, x: view.x, y: view.y });
      }}
      onPointerMove={(event: PointerEvent<HTMLDivElement>) => {
        if (!drag) return;
        setView((current) => ({
          ...current,
          x: drag.x + event.clientX - drag.pointerX,
          y: drag.y + event.clientY - drag.pointerY,
        }));
      }}
      onPointerUp={() => setDrag(null)}
      onPointerCancel={() => setDrag(null)}
      onWheel={(event: WheelEvent<HTMLDivElement>) => {
        const factor = event.deltaY < 0 ? 1.15 : 1 / 1.15;
        setView((current) => ({ ...current, zoom: Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, current.zoom * factor)) }));
      }}
    >
      <div
        className="absolute top-0 left-0 origin-top-left"
        style={{ transform: `translate(${view.x}px, ${view.y}px) scale(${view.zoom})` }}
      >
        {tiles.map((tile) => (
          <img
            key={tile.file}
            src={source.fileUrl(tile.file)}
            alt=""
            loading="lazy"
            decoding="async"
            draggable={false}
            className="absolute max-w-none [image-rendering:pixelated]"
            style={{
              left: (tile.x - originX) * TILE_PIXELS,
              top: (tile.z - originZ) * TILE_PIXELS,
              width: TILE_PIXELS,
              height: TILE_PIXELS,
            }}
          />
        ))}
      </div>
      <div className="pointer-events-none absolute inset-0">
        {inDimension.map((player) => (
          <PlayerTag
            key={player.name}
            name={player.name}
            src={`${headBase}${player.head}`}
            x={view.x + (player.position[0] - originX * TILE_PIXELS) * view.zoom}
            y={view.y + (player.position[2] - originZ * TILE_PIXELS) * view.zoom}
          />
        ))}
      </div>
    </div>
  );
}
