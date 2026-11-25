// A browser cache of meshed cells, so a revisit skips fetching and meshing them. Entries live in IndexedDB. Once they pass the
// byte budget the oldest go first. Every operation fails soft, because without the cache the viewer just meshes again.
import type { MeshedCell } from "./cell-mesher";

const DATABASE = "obsidian-worldmap-meshes";
const DATABASE_VERSION = 2;
const MESHES = "meshes";
const RECORDS = "records";
export const BUDGET_BYTES = 128 * 1024 * 1024;

/** `cell` identifies the cell; `variant` identifies its mesh: the pack, the mesher build and the cell version. */
export interface MeshKey {
  cell: string;
  variant: string;
}

export interface CacheRecord {
  key: string;
  storedAt: number;
  bytes: number;
}

export interface MeshCache {
  get(mesh: MeshKey): Promise<MeshedCell | undefined>;
  put(mesh: MeshKey, meshed: MeshedCell): Promise<void>;
  close(): void;
}

export function openMeshCache(): MeshCache {
  const database = openDatabase();
  return {
    async get(mesh) {
      const db = await database;
      if (!db) {
        return undefined;
      }
      try {
        return await read(db, entryKey(mesh));
      } catch {
        return undefined;
      }
    },
    async put(mesh, meshed) {
      const db = await database;
      if (!db) {
        return;
      }
      try {
        await store(db, mesh, meshed);
        await trim(db);
      } catch {
        // The next visit simply meshes this cell again.
      }
    },
    close() {
      void database.then((db) => db?.close());
    },
  };
}

export function meshBytes(meshed: MeshedCell): number {
  return meshed.layers.reduce((sum, layer) => sum + viewBytes(layer.instances) + viewBytes(layer.explicit), 0);
}

function viewBytes(arrays: object): number {
  return Object.values(arrays).reduce<number>((sum, value) => sum + (ArrayBuffer.isView(value) ? value.buffer.byteLength : 0), 0);
}

/** The keys to remove, oldest first, until the records that remain fit the budget. */
export function planEviction(records: CacheRecord[], budget: number): string[] {
  let total = records.reduce((sum, record) => sum + record.bytes, 0);
  const doomed: string[] = [];
  for (const record of [...records].sort((a, b) => a.storedAt - b.storedAt)) {
    if (total <= budget) {
      break;
    }
    doomed.push(record.key);
    total -= record.bytes;
  }
  return doomed;
}

function entryKey({ cell, variant }: MeshKey): string {
  return `${cell}|${variant}`;
}

function openDatabase(): Promise<IDBDatabase | undefined> {
  if (typeof indexedDB === "undefined") {
    return Promise.resolve(undefined);
  }
  return new Promise((resolve) => {
    let request: IDBOpenDBRequest;
    try {
      request = indexedDB.open(DATABASE, DATABASE_VERSION);
    } catch {
      resolve(undefined);
      return;
    }
    // Version 2 changed the key layout, so the entries of version 1 can never be read again.
    request.onupgradeneeded = () => {
      const db = request.result;
      for (const name of Array.from(db.objectStoreNames)) {
        db.deleteObjectStore(name);
      }
      db.createObjectStore(MESHES);
      db.createObjectStore(RECORDS, { keyPath: "key" });
    };
    // A tab running an older build can hold the old version open; the cache is skipped for this page instead of waiting.
    request.onblocked = () => resolve(undefined);
    request.onsuccess = () => {
      const db = request.result;
      // Lets a newer build upgrade the database rather than being blocked by this page.
      db.onversionchange = () => db.close();
      resolve(db);
    };
    request.onerror = () => resolve(undefined);
  });
}

function read(db: IDBDatabase, key: string): Promise<MeshedCell | undefined> {
  return settle<MeshedCell | undefined>(db.transaction(MESHES, "readonly").objectStore(MESHES).get(key));
}

/** Stores a mesh, and drops the same cell's entries under other variants: an older build, pack or cell version. */
async function store(db: IDBDatabase, mesh: MeshKey, meshed: MeshedCell): Promise<void> {
  const key = entryKey(mesh);
  const prefix = `${mesh.cell}|`;
  const sameCell = await settle(db.transaction(RECORDS, "readonly").objectStore(RECORDS).getAllKeys(IDBKeyRange.bound(prefix, `${prefix}\uffff`)));
  await new Promise<void>((resolve, reject) => {
    const transaction = db.transaction([MESHES, RECORDS], "readwrite");
    for (const other of sameCell) {
      if (other !== key) {
        transaction.objectStore(MESHES).delete(other);
        transaction.objectStore(RECORDS).delete(other);
      }
    }
    transaction.objectStore(MESHES).put(meshed, key);
    transaction.objectStore(RECORDS).put({ key, storedAt: Date.now(), bytes: meshBytes(meshed) } satisfies CacheRecord);
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}

async function trim(db: IDBDatabase): Promise<void> {
  const records = await settle<CacheRecord[]>(db.transaction(RECORDS, "readonly").objectStore(RECORDS).getAll());
  const doomed = planEviction(records, BUDGET_BYTES);
  if (doomed.length === 0) {
    return;
  }
  await new Promise<void>((resolve, reject) => {
    const transaction = db.transaction([MESHES, RECORDS], "readwrite");
    for (const key of doomed) {
      transaction.objectStore(MESHES).delete(key);
      transaction.objectStore(RECORDS).delete(key);
    }
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}

function settle<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}
