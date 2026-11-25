// A browser cache of meshed cells, so a revisit skips fetching and meshing them. Entries live in IndexedDB. Once they pass the
// byte budget the oldest go first. Every operation fails soft, because without the cache the viewer just meshes again.
import type { MeshedCell } from "./cell-mesher";

const DATABASE = "obsidian-worldmap-meshes";
const DATABASE_VERSION = 1;
const MESHES = "meshes";
const RECORDS = "records";
export const BUDGET_BYTES = 128 * 1024 * 1024;

export interface CacheRecord {
  key: string;
  storedAt: number;
  bytes: number;
}

export interface MeshCache {
  get(key: string): Promise<MeshedCell | undefined>;
  put(key: string, meshed: MeshedCell): Promise<void>;
  close(): void;
}

export function openMeshCache(): MeshCache {
  const database = openDatabase();
  return {
    async get(key) {
      const db = await database;
      if (!db) {
        return undefined;
      }
      try {
        return await read(db, key);
      } catch {
        return undefined;
      }
    },
    async put(key, meshed) {
      const db = await database;
      if (!db) {
        return;
      }
      try {
        await store(db, key, meshed);
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
    request.onupgradeneeded = () => {
      request.result.createObjectStore(MESHES);
      request.result.createObjectStore(RECORDS, { keyPath: "key" });
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => resolve(undefined);
  });
}

function read(db: IDBDatabase, key: string): Promise<MeshedCell | undefined> {
  return settle<MeshedCell | undefined>(db.transaction(MESHES, "readonly").objectStore(MESHES).get(key));
}

/** Stores a cell's mesh under its key, and drops the entries for the same cell under other versions. */
async function store(db: IDBDatabase, key: string, meshed: MeshedCell): Promise<void> {
  const group = key.slice(0, key.lastIndexOf("|") + 1);
  const siblings = await settle(db.transaction(RECORDS, "readonly").objectStore(RECORDS).getAllKeys(IDBKeyRange.bound(group, `${group}￿`)));
  await new Promise<void>((resolve, reject) => {
    const transaction = db.transaction([MESHES, RECORDS], "readwrite");
    for (const sibling of siblings) {
      if (sibling !== key) {
        transaction.objectStore(MESHES).delete(sibling);
        transaction.objectStore(RECORDS).delete(sibling);
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
