import { NearestFilter, NearestMipmapNearestFilter, NoColorSpace, type Scene, type Texture, TextureLoader, Vector2, type Vector3 } from "three";
import type { CellCoord, MapManifest, MapSource, SectionEntry } from "./map-source";
import { CellMesher, MESH_VERSION, MesherClosedError, type MeshedCell, workerCount } from "./structure/cell-mesher";
import { type CellMaterials, type CellObject, buildCell, createCellMaterials } from "./structure/cell-scene";
import { type MeshCache, type MeshKey, openMeshCache } from "./structure/mesh-cache";
import { type BiomeJson, type BlockTableJson, StaleTablesError } from "./structure/tables";

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
const FETCH_TIMEOUT_MS = 20_000;

type CellState = "idle" | "queued" | "loading";

interface Cell {
  coord: CellCoord;
  version: number;
  /** Bumped when a result still in flight is out of date, so it is dropped when it arrives. */
  generation: number;
  state: CellState;
  retryAt: number;
  /** The mesh on screen. It stays there until its replacement is ready. */
  object?: CellObject;
  /** The mesh on screen was built from an older version, so a replacement is due. */
  stale: boolean;
  failed: boolean;
}

export interface StreamStats {
  loaded: number;
  loading: number;
  queued: number;
  failed: number;
  busy: number;
  waiting: number;
}

export interface CellStreamOptions {
  scene: Scene;
  instanceId: number;
  /** The camera's position in blocks. Cells are chosen by their distance from it. */
  position: Vector3;
  onProgress(message: string, busy: boolean): void;
  /** A bundle named ids the loaded tables lack, so the manifest is read again. */
  onStaleTables(): void;
}

/**
 * Streams one dimension's cells into the scene around the camera. A mesh stays on screen until its replacement is built, so a
 * refresh, a table swap or a new pack never empties the scene. Only cells that leave the map are removed.
 */
export class CellStream {
  private readonly cells = new Map<string, Cell>();
  private readonly meshCache: MeshCache = openMeshCache();
  private manifest?: MapManifest;
  private atlas?: Texture;
  private atlasKey?: string;
  private atlasPending?: string;
  private materials?: CellMaterials;
  private mesher?: CellMesher;
  private tablesWanted?: string;
  private tablesLoaded?: string;
  private inFlight = 0;
  private reported = "";
  private disposed = false;

  constructor(
    private readonly options: CellStreamOptions,
    private readonly source: MapSource,
    private readonly folder: string,
  ) {
    document.addEventListener("visibilitychange", this.onVisibility);
  }

  /** Takes a manifest: which cells exist, and which tables, atlas and meshes they need now. */
  apply(next: MapManifest): void {
    const packChanged = this.manifest !== undefined && this.manifest.packKey !== next.packKey;
    this.manifest = next;
    const versions = new Map(next.tiles.map((tile) => [`${tile.x},${tile.z}`, tile.version]));
    const coords = new Map<string, CellCoord>();
    for (const section of next.sections) {
      const coord = cellOf(section);
      coords.set(coord.join(","), coord);
    }
    for (const [key, cell] of this.cells) {
      if (!coords.has(key)) {
        cell.generation++;
        this.release(cell);
        this.cells.delete(key);
      }
    }
    for (const [key, coord] of coords) {
      const version = versions.get(regionOf(coord)) ?? 0;
      const existing = this.cells.get(key);
      if (!existing) {
        this.cells.set(key, { coord, version, generation: 0, state: "idle", retryAt: 0, stale: false, failed: false });
      } else if (existing.version !== version || packChanged) {
        existing.version = version;
        existing.generation++;
        existing.stale = existing.object !== undefined;
        existing.retryAt = 0;
        if (existing.state === "loading") existing.state = "idle";
      }
    }
    const tables = tablesKeyOf(next);
    if (this.tablesWanted !== tables) this.loadTables(tables);
    if (this.atlasKey !== next.packKey && this.atlasPending !== next.packKey) this.loadAtlas(next);
    this.pump();
  }

  /** Called every frame: releases cells that left the view, queues cells that entered it, and starts loads. */
  refresh(): void {
    if (!this.manifest) return;
    const now = performance.now();
    for (const cell of this.cells.values()) {
      const distance = distanceToCell(cell.coord, this.options.position);
      if (cell.object && distance > LOAD_DISTANCE + UNLOAD_MARGIN) {
        this.release(cell);
      } else if (cell.state === "queued" && distance > LOAD_DISTANCE) {
        cell.state = "idle";
      } else if (cell.state === "idle" && distance <= LOAD_DISTANCE && now >= cell.retryAt && (!cell.object || cell.stale)) {
        cell.state = "queued";
      }
    }
    this.pump();
    this.report("Streaming");
  }

  stats(): StreamStats {
    const activity = this.mesher?.activity() ?? { busy: 0, queued: 0 };
    const stats: StreamStats = { loaded: 0, loading: this.inFlight, queued: 0, failed: 0, busy: activity.busy, waiting: activity.queued };
    for (const cell of this.cells.values()) {
      if (cell.object) stats.loaded++;
      if (cell.state === "queued") stats.queued++;
      if (cell.failed) stats.failed++;
    }
    return stats;
  }

  dispose(): void {
    this.disposed = true;
    document.removeEventListener("visibilitychange", this.onVisibility);
    for (const cell of this.cells.values()) {
      cell.generation++;
      this.release(cell);
    }
    this.cells.clear();
    this.mesher?.dispose();
    this.meshCache.close();
    this.atlas?.dispose();
    if (this.materials) disposeMaterials(this.materials);
  }

  private readonly onVisibility = (): void => {
    if (document.hidden) {
      this.mesher?.pause();
      return;
    }
    this.mesher?.resume();
    this.pump();
  };

  private pump(): void {
    const manifest = this.manifest;
    const materials = this.materials;
    const mesher = this.mesher;
    if (document.hidden || !manifest || !materials || !mesher) return;
    if (this.tablesLoaded !== tablesKeyOf(manifest) || this.atlasKey !== manifest.packKey) return;
    while (this.inFlight < MAX_CONCURRENT_LOADS) {
      const next = this.nearestQueued();
      if (!next) return;
      next.state = "loading";
      this.inFlight++;
      this.load(next, mesher, materials, manifest.packKey);
    }
  }

  private nearestQueued(): Cell | undefined {
    let best: Cell | undefined;
    let bestDistance = Infinity;
    for (const cell of this.cells.values()) {
      if (cell.state !== "queued") continue;
      const distance = distanceToCell(cell.coord, this.options.position);
      if (distance < bestDistance) {
        best = cell;
        bestDistance = distance;
      }
    }
    return best;
  }

  private load(cell: Cell, mesher: CellMesher, materials: CellMaterials, packKey: string): void {
    const generation = cell.generation;
    const version = cell.version;
    const id = cell.coord.join(",");
    const key = meshKey(this.folder, this.options.instanceId, cell.coord, packKey, version);
    this.meshCache
      .get(key)
      .then((cached) => cached ?? this.fetchMesh(cell, version, mesher, key))
      .then((meshed) => {
        if (this.disposed || cell.generation !== generation) return;
        cell.state = "idle";
        if (distanceToCell(cell.coord, this.options.position) > LOAD_DISTANCE + UNLOAD_MARGIN) return;
        this.attach(cell, meshed, materials);
        cell.failed = false;
        this.report("Streaming");
      })
      .catch((error: unknown) => {
        if (this.disposed || cell.generation !== generation) return;
        cell.state = "idle";
        if (error instanceof MesherClosedError) {
          cell.retryAt = 0;
          return;
        }
        cell.retryAt = performance.now() + RETRY_MS;
        cell.failed = true;
        if (error instanceof StaleTablesError) this.options.onStaleTables();
        console.error(`could not load cell ${id}`, error);
      })
      .finally(() => {
        this.inFlight--;
      });
  }

  private fetchMesh(cell: Cell, version: number, mesher: CellMesher, key: MeshKey): Promise<MeshedCell> {
    return fetch(this.source.structureCellUrl(cell.coord, version), { signal: AbortSignal.timeout(FETCH_TIMEOUT_MS) })
      .then((response) => {
        if (!response.ok) {
          throw new Error(`cell ${cell.coord.join(",")} failed with ${response.status}`);
        }
        return response.arrayBuffer();
      })
      .then((bundle) => mesher.mesh(bundle))
      .then((meshed) => {
        this.meshCache.put(key, meshed);
        return meshed;
      });
  }

  /** Swaps the mesh in one step: the new object is added before the old one is removed, within the same task. */
  private attach(cell: Cell, meshed: MeshedCell, materials: CellMaterials): void {
    const previous = cell.object;
    const object = buildCell(meshed.cell, meshed.layers, materials);
    this.options.scene.add(object.group);
    cell.object = object;
    cell.stale = false;
    if (previous) {
      this.options.scene.remove(previous.group);
      previous.dispose();
    }
  }

  private release(cell: Cell): void {
    if (cell.object) {
      this.options.scene.remove(cell.object.group);
      cell.object.dispose();
      cell.object = undefined;
    }
    cell.stale = false;
  }

  /** Builds a mesher from the tables of one version. Cells mesh only once the tables match the manifest's version. */
  private loadTables(key: string): void {
    this.tablesWanted = key;
    Promise.all([
      fetchJson<BlockTableJson>(this.source.fileUrl(`${this.folder}/structure/blocks.json`, key)),
      fetchJson<BiomeJson[]>(this.source.fileUrl(`${this.folder}/structure/biomes.json`, key)),
    ])
      .then(([blocks, biomes]) => {
        if (this.disposed || this.tablesWanted !== key) return;
        this.mesher?.dispose();
        this.mesher = new CellMesher(blocks, biomes, workerCount());
        if (document.hidden) this.mesher.pause();
        this.tablesLoaded = key;
        this.pump();
      })
      .catch((error: unknown) => {
        if (!this.disposed) this.options.onProgress(`Could not load the map: ${String(error)}`, false);
      });
  }

  /** The atlas is shown only once it has loaded, so no cell draws with a texture that is still empty. */
  private loadAtlas(next: MapManifest): void {
    const key = next.packKey;
    const width = next.atlasWidth;
    const height = next.atlasHeight;
    this.atlasPending = key;
    const texture = new TextureLoader().load(this.source.fileUrl(next.atlas, key), () => {
      if (this.disposed || this.atlasPending !== key) {
        texture.dispose();
        return;
      }
      this.atlasPending = undefined;
      this.atlasKey = key;
      const previous = this.atlas;
      this.atlas = texture;
      if (this.materials) {
        bindAtlas(this.materials, texture, width, height);
      } else {
        this.materials = createCellMaterials(texture, width, height);
      }
      previous?.dispose();
      this.pump();
    });
    texture.colorSpace = NoColorSpace;
    texture.magFilter = NearestFilter;
    texture.minFilter = NearestMipmapNearestFilter;
    texture.generateMipmaps = true;
    texture.flipY = false;
  }

  private report(message: string): void {
    const busy = this.inFlight > 0;
    const key = `${message}:${busy}`;
    if (this.disposed || key === this.reported) return;
    this.reported = key;
    this.options.onProgress(message, busy);
  }
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

/** Block and biome tables change with every render that adds a state or a biome, not only with the pack, so both name them. */
function tablesKeyOf(manifest: MapManifest): string {
  return `${manifest.packKey}.${manifest.tablesVersion ?? "0"}`;
}

function meshKey(folder: string, instanceId: number, coord: CellCoord, packKey: string, version: number): MeshKey {
  return {
    cell: [instanceId, folder, coord.join(",")].join("|"),
    variant: [packKey, MESH_VERSION, version].join("|"),
  };
}

function bindAtlas(materials: CellMaterials, texture: Texture, width: number, height: number): void {
  for (const material of materials.instanced) {
    material.uniforms.uAtlas.value = texture;
    material.uniforms.uAtlasSize.value = new Vector2(width, height);
  }
  for (const material of materials.explicit) {
    material.uniforms.uAtlas.value = texture;
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
