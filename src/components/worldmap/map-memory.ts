/**
 * What the built-in map remembers for each instance: the view it was on, the dimension, and for each dimension the block
 * the views centre on, the 3D camera's height and facing, and the top-down zoom. Changes land in memory at once, and reach
 * local storage a moment later, so a camera that moves every frame does not write every frame.
 */

export type MapView = "top" | "free";

export interface DimensionMemory {
  /** Block x and z at the centre of the view. The top-down view and the 3D camera share them. */
  x?: number;
  z?: number;
  /** The 3D camera's height, and where it faces. */
  y?: number;
  yaw?: number;
  pitch?: number;
  /** Top-down zoom, in screen pixels per block. */
  zoom?: number;
}

export interface MapMemory {
  view?: MapView;
  folder?: string;
  dimensions: Record<string, DimensionMemory>;
}

const WRITE_DELAY_MS = 300;
const NUMBER_KEYS: (keyof DimensionMemory)[] = ["x", "y", "z", "yaw", "pitch", "zoom"];

const memories = new Map<number, MapMemory>();
const pendingWrites = new Map<number, number>();

export function mapMemory(instanceId: number): MapMemory {
  let memory = memories.get(instanceId);
  if (!memory) {
    memory = read(instanceId);
    memories.set(instanceId, memory);
  }
  return memory;
}

export function dimensionMemory(instanceId: number, folder: string): DimensionMemory {
  return mapMemory(instanceId).dimensions[folder] ?? {};
}

export function rememberView(instanceId: number, patch: { view?: MapView; folder?: string }): void {
  Object.assign(mapMemory(instanceId), patch);
  scheduleWrite(instanceId);
}

export function rememberDimension(instanceId: number, folder: string, patch: DimensionMemory): void {
  const memory = mapMemory(instanceId);
  memory.dimensions[folder] = { ...memory.dimensions[folder], ...patch };
  scheduleWrite(instanceId);
}

if (typeof window !== "undefined") {
  window.addEventListener("pagehide", () => {
    for (const instanceId of [...pendingWrites.keys()]) {
      flush(instanceId);
    }
  });
}

function scheduleWrite(instanceId: number): void {
  if (pendingWrites.has(instanceId)) {
    return;
  }
  pendingWrites.set(
    instanceId,
    window.setTimeout(() => flush(instanceId), WRITE_DELAY_MS),
  );
}

function flush(instanceId: number): void {
  const timer = pendingWrites.get(instanceId);
  if (timer !== undefined) {
    window.clearTimeout(timer);
    pendingWrites.delete(instanceId);
  }
  const memory = memories.get(instanceId);
  if (!memory) {
    return;
  }
  try {
    window.localStorage.setItem(storageKey(instanceId), JSON.stringify(memory));
  } catch {
    // Storage can be full or switched off. The memory still holds for this visit.
  }
}

function read(instanceId: number): MapMemory {
  try {
    const stored = window.localStorage.getItem(storageKey(instanceId));
    return stored ? sanitize(JSON.parse(stored)) : { dimensions: {} };
  } catch {
    return { dimensions: {} };
  }
}

function storageKey(instanceId: number): string {
  return `obsidian.worldmap.${instanceId}`;
}

/** Keeps only the values this version understands, so an older or damaged entry cannot break the map. */
function sanitize(value: unknown): MapMemory {
  const memory: MapMemory = { dimensions: {} };
  if (!isRecord(value)) {
    return memory;
  }
  if (value.view === "top" || value.view === "free") {
    memory.view = value.view;
  }
  if (typeof value.folder === "string") {
    memory.folder = value.folder;
  }
  if (isRecord(value.dimensions)) {
    for (const [folder, entry] of Object.entries(value.dimensions)) {
      if (isRecord(entry)) {
        memory.dimensions[folder] = numbersOf(entry);
      }
    }
  }
  return memory;
}

function numbersOf(entry: Record<string, unknown>): DimensionMemory {
  const remembered: DimensionMemory = {};
  for (const key of NUMBER_KEYS) {
    const value = entry[key];
    if (typeof value === "number" && Number.isFinite(value)) {
      remembered[key] = value;
    }
  }
  return remembered;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
