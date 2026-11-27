import { useEffect, useRef, useState } from "react";
import {
  Color,
  NearestFilter,
  PerspectiveCamera,
  Scene,
  SRGBColorSpace,
  Texture,
  TextureLoader,
  Vector2,
  Vector3,
  WebGLRenderer,
} from "three";
import type { LivePlayer } from "../../api/worldmap";
import { applyFocus } from "./follow-camera";
import { FreeFlyCamera } from "./free-fly-camera";
import type { MapFocus } from "./map-focus";
import { dimensionMemory, rememberDimension } from "./map-memory";
import type { CellCoord, MapManifest, MapSource, SectionEntry } from "./map-source";
import type { MapStats } from "./map-stats";
import { PlayerMarkers } from "./player-markers";
import { PlayerTrack } from "./player-track";
import { CellMesher, MESH_VERSION, MesherClosedError, type MeshedCell, workerCount } from "./structure/cell-mesher";
import { type CellMaterials, type CellObject, buildCell, createCellMaterials } from "./structure/cell-scene";
import { type MeshKey, openMeshCache } from "./structure/mesh-cache";
import { type BiomeJson, type BlockTableJson, StaleTablesError } from "./structure/tables";
import type { ViewPlace } from "./use-viewer-heartbeat";

/** Cells whose nearest point is closer than this to the camera are loaded, in blocks. */
const LOAD_DISTANCE = 160;
/** Loaded cells farther than LOAD_DISTANCE plus this margin are released. */
const UNLOAD_MARGIN = 96;
/** A cell is four chunks and four sections on each side: 64 blocks, the grid the server bundles by. */
const CELL_BLOCKS = 64;
/** Cells per region side, since a region is 32 chunks. */
const CELLS_PER_REGION = 8;
const MAX_CONCURRENT_LOADS = 6;
const RETRY_MS = 30_000;
const CELL_TIMEOUT_MS = 20_000;
/** How often the camera's place is reported while it moves, in seconds. */
const REMEMBER_EVERY = 0.25;
/** How often the statistics are published, in seconds. */
const STATS_EVERY = 0.5;
const DEFAULT_PITCH = -0.45;
const LOOK_DOWN_HEIGHT = 40;
const SKY = new Color(0x9ec9f5);

type CellState = "idle" | "queued" | "loading" | "loaded";

interface Cell {
  coord: CellCoord;
  version: number;
  /** Bumped when the cell's contents change, so a response that is still in flight is dropped. */
  generation: number;
  state: CellState;
  retryAt: number;
  object?: CellObject;
}

interface Progress {
  message: string;
  /** Cells are being fetched right now. */
  busy: boolean;
}

export interface WorldMapViewerProps {
  source: MapSource;
  instanceId: number;
  folder: string;
  /** Changes when the map changes. The viewer fetches the manifest again and streams only the cells that changed. */
  revision: number;
  players: readonly LivePlayer[];
  skinBase: string;
  focus: MapFocus | null;
  stats: MapStats;
  onPlace: (place: ViewPlace) => void;
  /** The keys or the mouse moved the camera by hand, so a followed player is let go. */
  onInput: () => void;
}

/** A free-fly 3D view of the cells, streamed in around the camera. Each cell is meshed in a worker from its structure. */
export function WorldMapViewer({
  source,
  instanceId,
  folder,
  revision,
  players,
  skinBase,
  focus,
  stats,
  onPlace,
  onInput,
}: WorldMapViewerProps) {
  const playersRef = useRef(players);
  playersRef.current = players;
  const focusRef = useRef(focus);
  focusRef.current = focus;
  const placeRef = useRef(onPlace);
  placeRef.current = onPlace;
  const inputRef = useRef(onInput);
  inputRef.current = onInput;
  const [track] = useState(() => new PlayerTrack());
  const markersRef = useRef<PlayerMarkers | null>(null);
  const reloadRef = useRef<(() => void) | null>(null);
  const revisionRef = useRef(revision);
  const containerRef = useRef<HTMLDivElement>(null);
  const [progress, setProgress] = useState<Progress>({ message: "Loading the map", busy: true });

  useEffect(() => {
    track.record(players, performance.now());
    markersRef.current?.update(players);
  }, [players, track]);

  useEffect(() => {
    markersRef.current?.hide(focus?.follow === "head" ? focus.name : null);
  }, [focus]);

  useEffect(() => {
    if (revisionRef.current === revision) {
      return;
    }
    revisionRef.current = revision;
    reloadRef.current?.();
  }, [revision]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return undefined;
    }
    let disposed = false;
    let raf = 0;
    let last = performance.now();
    let inFlight = 0;
    let placed = false;
    let sinceRemembered = 0;
    let applied = -1;
    let manifest: MapManifest | undefined;
    let atlas: Texture | undefined;
    let atlasKey: string | undefined;
    let materials: CellMaterials | undefined;
    let mesher: CellMesher | undefined;
    let tablesWanted: string | undefined;
    let tablesLoaded: string | undefined;
    const failed = new Set<string>();
    let reported = "";
    const cells = new Map<string, Cell>();
    const meshCache = openMeshCache();

    const renderer = new WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(SKY);
    renderer.domElement.style.display = "block";
    container.appendChild(renderer.domElement);
    const scene = new Scene();
    const camera = new PerspectiveCamera(70, 1, 0.5, 4000);
    const fly = new FreeFlyCamera();
    const detachInput = fly.attach(renderer.domElement, { onInput: () => inputRef.current() });
    const markers = new PlayerMarkers(skinBase);
    scene.add(markers.group);
    markers.update(playersRef.current);
    markersRef.current = markers;
    stats.set({ gpu: gpuName(renderer) });
    let frames = 0;
    let windowMs = 0;
    let scriptMs = 0;
    let sinceStats = 0;

    let lastPlace = "";
    const rememberPlace = () => {
      const place = [fly.position.x, fly.position.y, fly.position.z, fly.yaw, fly.pitch].join(",");
      if (place === lastPlace) {
        return;
      }
      lastPlace = place;
      placeRef.current({ x: fly.position.x, y: fly.position.y, z: fly.position.z });
      rememberDimension(instanceId, folder, {
        x: fly.position.x,
        y: fly.position.y,
        z: fly.position.z,
        yaw: fly.yaw,
        pitch: fly.pitch,
      });
    };

    let viewportHeight = 1;
    const resize = () => {
      const width = Math.max(1, container.clientWidth);
      const height = Math.max(1, container.clientHeight);
      viewportHeight = height;
      renderer.setSize(width, height);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(container);

    const loadedCells = () => {
      let count = 0;
      for (const cell of cells.values()) {
        if (cell.state === "loaded") count++;
      }
      return count;
    };

    const publishStats = () => {
      const { busy, queued: waiting } = mesher?.activity() ?? { busy: 0, queued: 0 };
      let queued = 0;
      for (const cell of cells.values()) {
        if (cell.state === "queued") queued++;
      }
      stats.set({
        fps: (frames * 1000) / Math.max(windowMs, 1),
        frameMs: windowMs / Math.max(frames, 1),
        scriptMs: scriptMs / Math.max(frames, 1),
        drawCalls: renderer.info.render.calls,
        triangles: renderer.info.render.triangles,
        textures: renderer.info.memory.textures,
        geometries: renderer.info.memory.geometries,
        heapMb: heapMegabytes(),
        cellsLoaded: loadedCells(),
        cellsQueued: queued,
        cellsLoading: inFlight,
        cellsFailed: failed.size,
        workersBusy: busy,
        workersQueued: waiting,
      });
      frames = 0;
      windowMs = 0;
      scriptMs = 0;
      sinceStats = 0;
    };

    const report = (message: string) => {
      const busy = inFlight > 0;
      const key = `${message}:${busy}`;
      if (disposed || key === reported) {
        return;
      }
      reported = key;
      setProgress({ message, busy });
    };

    const release = (cell: Cell) => {
      if (cell.object) {
        scene.remove(cell.object.group);
        cell.object.dispose();
        cell.object = undefined;
      }
      cell.state = "idle";
    };

    const attach = (cell: Cell, meshed: MeshedCell, cellMaterials: CellMaterials) => {
      const object = buildCell(meshed.cell, meshed.layers, cellMaterials);
      scene.add(object.group);
      cell.object = object;
      cell.state = "loaded";
    };

    const meshKey = (cell: Cell, packKey: string): MeshKey => ({
      cell: [instanceId, folder, cell.coord.join(",")].join("|"),
      variant: [packKey, MESH_VERSION, cell.version].join("|"),
    });

    const fetchMesh = (cell: Cell, cellMesher: CellMesher, key: MeshKey): Promise<MeshedCell> =>
      fetch(source.structureCellUrl(cell.coord, cell.version), { signal: AbortSignal.timeout(CELL_TIMEOUT_MS) })
        .then((response) => {
          if (!response.ok) {
            throw new Error(`cell ${cell.coord.join(",")} failed with ${response.status}`);
          }
          return response.arrayBuffer();
        })
        .then((bundle) => cellMesher.mesh(bundle))
        .then((meshed) => {
          meshCache.put(key, meshed);
          return meshed;
        });

    const load = (cell: Cell, cellMesher: CellMesher, cellMaterials: CellMaterials, key: MeshKey) => {
      const generation = cell.generation;
      const id = cell.coord.join(",");
      meshCache
        .get(key)
        .then((cached) => cached ?? fetchMesh(cell, cellMesher, key))
        .then((meshed) => {
          if (disposed || cell.generation !== generation) {
            return;
          }
          if (distanceToCell(cell.coord, fly.position) > LOAD_DISTANCE + UNLOAD_MARGIN) {
            cell.state = "idle";
            return;
          }
          attach(cell, meshed, cellMaterials);
          failed.delete(id);
          report("Streaming");
        })
        .catch((error: unknown) => {
          if (disposed || cell.generation !== generation) {
            return;
          }
          cell.state = "idle";
          if (error instanceof MesherClosedError) {
            cell.retryAt = 0;
            return;
          }
          cell.retryAt = performance.now() + RETRY_MS;
          failed.add(id);
          if (error instanceof StaleTablesError) {
            reloadRef.current?.();
          }
          console.error(`could not load cell ${id}`, error);
        })
        .finally(() => {
          inFlight--;
        });
    };

    const nearestQueued = (): Cell | undefined => {
      let best: Cell | undefined;
      let bestDistance = Infinity;
      for (const cell of cells.values()) {
        if (cell.state !== "queued") continue;
        const distance = distanceToCell(cell.coord, fly.position);
        if (distance < bestDistance) {
          best = cell;
          bestDistance = distance;
        }
      }
      return best;
    };

    const pump = () => {
      if (document.hidden || !manifest || !materials || !mesher || tablesLoaded !== tablesKeyOf(manifest)) {
        return;
      }
      const cellMesher = mesher;
      const cellMaterials = materials;
      const packKey = manifest.packKey;
      while (inFlight < MAX_CONCURRENT_LOADS) {
        const next = nearestQueued();
        if (!next) {
          return;
        }
        next.state = "loading";
        inFlight++;
        load(next, cellMesher, cellMaterials, meshKey(next, packKey));
      }
    };

    const onVisibilityChange = () => {
      if (document.hidden) {
        mesher?.pause();
        return;
      }
      mesher?.resume();
      pump();
    };
    document.addEventListener("visibilitychange", onVisibilityChange);

    const refresh = () => {
      if (!manifest) {
        return;
      }
      const now = performance.now();
      for (const cell of cells.values()) {
        const distance = distanceToCell(cell.coord, fly.position);
        if (cell.state === "loaded" && distance > LOAD_DISTANCE + UNLOAD_MARGIN) {
          release(cell);
        } else if (cell.state === "queued" && distance > LOAD_DISTANCE) {
          cell.state = "idle";
        } else if (cell.state === "idle" && distance <= LOAD_DISTANCE && now >= cell.retryAt) {
          cell.state = "queued";
        }
      }
      pump();
      report("Streaming");
    };

    const placeLookingDown = (loadedManifest: MapManifest) => {
      if (loadedManifest.sections.length === 0) {
        return;
      }
      const xs = loadedManifest.sections.flatMap((entry) => [entry.x * 16, entry.x * 16 + 16]);
      const zs = loadedManifest.sections.flatMap((entry) => [entry.z * 16, entry.z * 16 + 16]);
      fly.position.set(
        (Math.min(...xs) + Math.max(...xs)) / 2,
        heightAboveTop(loadedManifest),
        (Math.min(...zs) + Math.max(...zs)) / 2 + 80,
      );
      fly.yaw = 0;
      fly.pitch = DEFAULT_PITCH;
    };

    const placeInitially = (loadedManifest: MapManifest) => {
      const remembered = dimensionMemory(instanceId, folder);
      if (remembered.x === undefined || remembered.z === undefined) {
        placeLookingDown(loadedManifest);
        return;
      }
      fly.position.set(remembered.x, remembered.y ?? heightAboveTop(loadedManifest), remembered.z);
      fly.yaw = remembered.yaw ?? 0;
      fly.pitch = remembered.pitch ?? DEFAULT_PITCH;
    };

    const loadTables = (key: string) => {
      tablesWanted = key;
      Promise.all([
        fetchJson<BlockTableJson>(source.fileUrl(`${folder}/structure/blocks.json`, key)),
        fetchJson<BiomeJson[]>(source.fileUrl(`${folder}/structure/biomes.json`, key)),
      ])
        .then(([blocks, biomes]) => {
          if (disposed || tablesWanted !== key) {
            return;
          }
          mesher?.dispose();
          mesher = new CellMesher(blocks, biomes, workerCount());
          if (document.hidden) mesher.pause();
          tablesLoaded = key;
          pump();
        })
        .catch((error: unknown) => {
          if (!disposed) {
            setProgress({ message: `Could not load the map: ${String(error)}`, busy: false });
          }
        });
    };

    const applyManifest = (next: MapManifest) => {
      manifest = next;
      const versions = new Map(next.tiles.map((tile) => [`${tile.x},${tile.z}`, tile.version]));
      const coords = new Map<string, CellCoord>();
      for (const section of next.sections) {
        const coord = cellOf(section);
        coords.set(coord.join(","), coord);
      }
      for (const [key, cell] of cells) {
        if (!coords.has(key)) {
          cell.generation++;
          release(cell);
          cells.delete(key);
          failed.delete(key);
        }
      }
      for (const [key, coord] of coords) {
        const version = versions.get(regionOf(coord)) ?? 0;
        const existing = cells.get(key);
        if (!existing) {
          cells.set(key, { coord, version, generation: 0, state: "idle", retryAt: 0, object: undefined });
        } else if (existing.version !== version) {
          release(existing);
          existing.version = version;
          existing.generation++;
          existing.retryAt = 0;
        }
      }
      if (!atlas || atlasKey !== next.packKey) {
        const replacement = loadAtlas(source.fileUrl(next.atlas, next.packKey));
        atlas?.dispose();
        atlas = replacement;
        atlasKey = next.packKey;
        if (materials) {
          bindAtlas(materials, replacement, next.atlasWidth, next.atlasHeight);
        } else {
          materials = createCellMaterials(replacement, next.atlasWidth, next.atlasHeight);
        }
      }
      const tables = tablesKeyOf(next);
      if (tablesWanted !== tables) {
        loadTables(tables);
      }
      if (!placed) {
        placeInitially(next);
        placed = true;
      }
      report("Streaming");
    };

    const loadManifest = () => {
      source
        .loadManifest()
        .then((loaded) => {
          if (!disposed) applyManifest(loaded);
        })
        .catch((error: unknown) => {
          if (!disposed) {
            setProgress({ message: `Could not load the map: ${String(error)}`, busy: false });
          }
        });
    };
    reloadRef.current = loadManifest;
    loadManifest();

    const tick = (now: number) => {
      if (disposed) {
        return;
      }
      const started = performance.now();
      const real = Math.max(0, now - last);
      const seconds = Math.min(real / 1000, 0.1);
      last = now;
      if (placed) {
        const current = focusRef.current;
        applied = applyFocus(fly, current, current ? track.pose(current.name, now) : undefined, applied);
      }
      fly.update(seconds, camera);
      refresh();
      if (placed) {
        sinceRemembered += seconds;
        if (sinceRemembered >= REMEMBER_EVERY) {
          sinceRemembered = 0;
          rememberPlace();
        }
      }
      markers.frame(camera, viewportHeight, (name) => track.pose(name, now));
      renderer.render(scene, camera);
      frames++;
      windowMs += real;
      scriptMs += performance.now() - started;
      sinceStats += seconds;
      if (sinceStats >= STATS_EVERY) {
        publishStats();
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);

    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
      observer.disconnect();
      document.removeEventListener("visibilitychange", onVisibilityChange);
      detachInput();
      if (placed) {
        rememberPlace();
      }
      for (const cell of cells.values()) {
        release(cell);
      }
      cells.clear();
      mesher?.dispose();
      meshCache.close();
      reloadRef.current = null;
      markersRef.current = null;
      markers.dispose();
      atlas?.dispose();
      if (materials) {
        disposeMaterials(materials);
      }
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [source, skinBase, instanceId, folder, track, stats]);

  const streaming = progress.message === "Streaming";
  return (
    <div className="absolute inset-0 overflow-hidden bg-black">
      <div ref={containerRef} className="absolute inset-0" />
      {!streaming && (
        <div className="pointer-events-none absolute top-3 right-3 flex items-center gap-2 rounded-control border border-line bg-canvas/80 px-2.5 py-1.5 font-mono text-[11px] text-ink-2 shadow-lift backdrop-blur-md">
          {progress.busy && <span className="size-1.5 animate-pulse rounded-full bg-accent" />}
          {progress.message}
        </div>
      )}
      <div className="pointer-events-none absolute bottom-3 left-3 flex max-w-[calc(100%-1.5rem)] flex-wrap items-center gap-x-3 gap-y-1 rounded-control border border-line bg-canvas/80 px-2.5 py-1.5 text-[11px] text-ink-2 shadow-lift backdrop-blur-md">
        <Hint keys={["Right-drag"]} label="look" />
        <Hint keys={["Double-click"]} label="lock, Esc to leave" />
        <Hint keys={["W", "A", "S", "D"]} label="fly" />
        <Hint keys={["Space", "Shift"]} label="up, down" />
        <Hint keys={["Ctrl"]} label="faster" />
        <Hint keys={["Wheel"]} label="speed" />
      </div>
    </div>
  );
}

function Hint({ keys, label }: { keys: string[]; label: string }) {
  return (
    <span className="flex items-center gap-1">
      {keys.map((key) => (
        <kbd key={key} className="rounded border border-line-strong bg-input px-1 font-mono text-[10px] text-ink-1">
          {key}
        </kbd>
      ))}
      <span className="text-ink-3">{label}</span>
    </span>
  );
}

function cellOf(entry: SectionEntry): CellCoord {
  return [Math.floor(entry.x / 4), Math.floor(entry.y / 4), Math.floor(entry.z / 4)];
}

/** The tables change with every render that adds a block state or biome, not only with the pack, so they are named by both. */
function tablesKeyOf(manifest: MapManifest): string {
  return `${manifest.packKey}.${manifest.tablesVersion ?? "0"}`;
}

/** The region a cell lies in, as the `x,z` key of the manifest's tiles. */
function regionOf(coord: CellCoord): string {
  return `${Math.floor(coord[0] / CELLS_PER_REGION)},${Math.floor(coord[2] / CELLS_PER_REGION)}`;
}

/** Distance in blocks from `point` to the nearest face of a cell, or zero when the point is inside it. */
function distanceToCell(coord: CellCoord, point: Vector3): number {
  const gap = (value: number, low: number) => Math.max(low - value, 0, value - (low + CELL_BLOCKS));
  return Math.hypot(gap(point.x, coord[0] * CELL_BLOCKS), gap(point.y, coord[1] * CELL_BLOCKS), gap(point.z, coord[2] * CELL_BLOCKS));
}

/** The height that looks over the top of the map: the highest section, and a margin above it. */
function heightAboveTop(manifest: MapManifest): number {
  return Math.max(0, ...manifest.sections.map((entry) => entry.y * 16 + 16)) + LOOK_DOWN_HEIGHT;
}

function loadAtlas(url: string): Texture {
  const texture = new TextureLoader().load(url);
  texture.colorSpace = SRGBColorSpace;
  texture.magFilter = NearestFilter;
  texture.minFilter = NearestFilter;
  texture.generateMipmaps = false;
  texture.flipY = false;
  return texture;
}

function bindAtlas(materials: CellMaterials, texture: Texture, width: number, height: number): void {
  for (const material of materials.instanced) {
    material.uniforms.uAtlas.value = texture;
    material.uniforms.uAtlasSize.value = new Vector2(width, height);
  }
  for (const material of materials.explicit) {
    material.map = texture;
    material.needsUpdate = true;
  }
}

function disposeMaterials(materials: CellMaterials): void {
  for (const material of [...materials.instanced, ...materials.explicit]) {
    material.dispose();
  }
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${url} failed with ${response.status}`);
  }
  return (await response.json()) as T;
}

function gpuName(renderer: WebGLRenderer): string {
  const gl = renderer.getContext();
  const info = gl.getExtension("WEBGL_debug_renderer_info");
  return info ? String(gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) : "hidden by the browser";
}

function heapMegabytes(): number | null {
  const memory = (performance as Performance & { memory?: { usedJSHeapSize: number } }).memory;
  return memory ? memory.usedJSHeapSize / 1048576 : null;
}
