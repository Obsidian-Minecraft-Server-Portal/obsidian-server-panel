import { useEffect, useRef, useState } from "react";
import {
  BufferGeometry,
  Color,
  Group,
  Mesh,
  MeshBasicMaterial,
  NearestFilter,
  PerspectiveCamera,
  Scene,
  SRGBColorSpace,
  Texture,
  TextureLoader,
  Vector3,
  WebGLRenderer,
} from "three";
import type { LivePlayer } from "../../api/worldmap";
import { type BundleEntry, decodeCellBundle } from "./cell-bundle";
import { FreeFlyCamera } from "./free-fly-camera";
import { dimensionMemory, rememberDimension } from "./map-memory";
import type { CellCoord, MapManifest, MapSource, SectionEntry } from "./map-source";
import { decodeSectionMesh } from "./mesh-format";
import { PlayerMarkers } from "./player-markers";
import { buildLayerGeometry } from "./section-geometry";

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
/** How often the camera's place is written to memory while it moves, in seconds. */
const REMEMBER_EVERY = 0.25;
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
  sections: number;
  groups: Group[];
  geometries: BufferGeometry[];
}

interface Progress {
  message: string;
  loaded: number;
  total: number;
}

export interface WorldMapViewerProps {
  source: MapSource;
  instanceId: number;
  folder: string;
  /** Changes when the map changes. The viewer fetches the manifest again and streams only the cells that changed. */
  revision: number;
  players: readonly LivePlayer[];
  skinBase: string;
}

/** A free-fly 3D view of the section meshes, streamed in cell by cell around the camera. */
export function WorldMapViewer({ source, instanceId, folder, revision, players, skinBase }: WorldMapViewerProps) {
  const playersRef = useRef(players);
  playersRef.current = players;
  const markersRef = useRef<PlayerMarkers | null>(null);
  const reloadRef = useRef<(() => void) | null>(null);
  const revisionRef = useRef(revision);
  const containerRef = useRef<HTMLDivElement>(null);
  const [progress, setProgress] = useState<Progress>({ message: "Loading manifest", loaded: 0, total: 0 });

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
    let frame = 0;
    let last = performance.now();
    let inFlight = 0;
    let placed = false;
    let sinceRemembered = 0;
    let manifest: MapManifest | undefined;
    let atlas: Texture | undefined;
    let atlasKey: string | undefined;
    let materials: MeshBasicMaterial[] | undefined;
    let reported = "";
    const cells = new Map<string, Cell>();

    const renderer = new WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(SKY);
    renderer.domElement.style.display = "block";
    container.appendChild(renderer.domElement);
    const scene = new Scene();
    const camera = new PerspectiveCamera(70, 1, 0.5, 4000);
    const fly = new FreeFlyCamera();
    const detachInput = fly.attach(renderer.domElement);
    const markers = new PlayerMarkers(skinBase);
    scene.add(markers.group);
    markers.update(playersRef.current);
    markersRef.current = markers;

    let lastPlace = "";
    const rememberPlace = () => {
      const place = [fly.position.x, fly.position.y, fly.position.z, fly.yaw, fly.pitch].join(",");
      if (place === lastPlace) {
        return;
      }
      lastPlace = place;
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

    const loadedSections = () => {
      let count = 0;
      for (const cell of cells.values()) {
        if (cell.state === "loaded") count += cell.sections;
      }
      return count;
    };

    const report = (message: string) => {
      const loaded = loadedSections();
      const key = `${message}:${loaded}`;
      if (disposed || key === reported) {
        return;
      }
      reported = key;
      setProgress({ message, loaded, total: manifest?.sections.length ?? 0 });
    };

    const release = (cell: Cell) => {
      for (const group of cell.groups) {
        scene.remove(group);
      }
      cell.geometries.forEach((geometry) => geometry.dispose());
      cell.groups = [];
      cell.geometries = [];
      cell.sections = 0;
      cell.state = "idle";
    };

    const addCell = (cell: Cell, entries: BundleEntry[], layerMaterials: MeshBasicMaterial[]) => {
      const decoded = entries.map((entry) => ({ entry, mesh: decodeSectionMesh(entry.mesh) }));
      for (const { entry, mesh } of decoded) {
        const group = new Group();
        group.position.set(entry.x * 16, entry.y * 16, entry.z * 16);
        mesh.layers.forEach((layer, index) => {
          if (layer.quadCount === 0) {
            return;
          }
          const geometry = buildLayerGeometry(layer);
          cell.geometries.push(geometry);
          group.add(new Mesh(geometry, layerMaterials[index]));
        });
        scene.add(group);
        cell.groups.push(group);
      }
      cell.sections = decoded.length;
      cell.state = "loaded";
    };

    const load = (cell: Cell, layerMaterials: MeshBasicMaterial[]) => {
      const generation = cell.generation;
      fetch(source.cellUrl(cell.coord, cell.version))
        .then((response) => {
          if (!response.ok) {
            throw new Error(`cell ${cell.coord.join(",")} failed with ${response.status}`);
          }
          return response.arrayBuffer();
        })
        .then((buffer) => {
          if (disposed || cell.generation !== generation) {
            return;
          }
          if (distanceToCell(cell.coord, fly.position) > LOAD_DISTANCE + UNLOAD_MARGIN) {
            cell.state = "idle";
            return;
          }
          addCell(cell, decodeCellBundle(buffer), layerMaterials);
          report("Streaming");
        })
        .catch((error: unknown) => {
          if (disposed || cell.generation !== generation) {
            return;
          }
          cell.state = "idle";
          cell.retryAt = performance.now() + RETRY_MS;
          console.error(`could not load cell ${cell.coord.join(",")}`, error);
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
      if (!manifest || !materials) {
        return;
      }
      const layerMaterials = materials;
      while (inFlight < MAX_CONCURRENT_LOADS) {
        const next = nearestQueued();
        if (!next) {
          return;
        }
        next.state = "loading";
        inFlight++;
        load(next, layerMaterials);
      }
    };

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
      fly.position.set((Math.min(...xs) + Math.max(...xs)) / 2, heightAboveTop(loadedManifest), (Math.min(...zs) + Math.max(...zs)) / 2 + 80);
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
        }
      }
      for (const [key, coord] of coords) {
        const version = versions.get(regionOf(coord)) ?? 0;
        const existing = cells.get(key);
        if (!existing) {
          cells.set(key, { coord, version, generation: 0, state: "idle", retryAt: 0, sections: 0, groups: [], geometries: [] });
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
          for (const material of materials) {
            material.map = replacement;
            material.needsUpdate = true;
          }
        } else {
          materials = createMaterials(replacement);
        }
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
            setProgress({ message: `Could not load the map: ${String(error)}`, loaded: 0, total: 0 });
          }
        });
    };
    reloadRef.current = loadManifest;
    loadManifest();

    const tick = (now: number) => {
      if (disposed) {
        return;
      }
      const seconds = Math.min((now - last) / 1000, 0.1);
      last = now;
      fly.update(seconds, camera);
      refresh();
      if (placed) {
        sinceRemembered += seconds;
        if (sinceRemembered >= REMEMBER_EVERY) {
          sinceRemembered = 0;
          rememberPlace();
        }
      }
      markers.frame(camera, viewportHeight);
      renderer.render(scene, camera);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);

    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
      observer.disconnect();
      detachInput();
      if (placed) {
        rememberPlace();
      }
      for (const cell of cells.values()) {
        release(cell);
      }
      cells.clear();
      reloadRef.current = null;
      markersRef.current = null;
      markers.dispose();
      atlas?.dispose();
      materials?.forEach((material) => material.dispose());
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [source, skinBase, instanceId, folder]);

  useEffect(() => {
    markersRef.current?.update(players);
  }, [players]);

  return (
    <div style={{ position: "relative", width: "100%", height: "100%", overflow: "hidden", background: "#000" }}>
      <div ref={containerRef} style={{ position: "absolute", inset: 0 }} />
      <div
        style={{
          position: "absolute",
          left: 12,
          top: 12,
          padding: "6px 10px",
          font: "13px/1.5 system-ui, sans-serif",
          color: "#fff",
          background: "rgba(0, 0, 0, 0.55)",
          borderRadius: 6,
          pointerEvents: "none",
        }}
      >
        {progress.message} · {progress.loaded}/{progress.total} sections
        <br />
        Click the view to fly. WASD move along the view, Space and Shift up and down, Ctrl to go faster, Esc to release.
      </div>
    </div>
  );
}

function cellOf(entry: SectionEntry): CellCoord {
  return [Math.floor(entry.x / 4), Math.floor(entry.y / 4), Math.floor(entry.z / 4)];
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

function createMaterials(map: Texture): MeshBasicMaterial[] {
  return [
    new MeshBasicMaterial({ map, vertexColors: true }),
    new MeshBasicMaterial({ map, vertexColors: true, alphaTest: 0.5 }),
    new MeshBasicMaterial({ map, vertexColors: true, transparent: true, depthWrite: false }),
  ];
}
