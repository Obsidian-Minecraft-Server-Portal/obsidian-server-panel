// Main-thread side of the mesh workers: sends the tables to each worker once, then queues cell bundles and resolves each one
// with its mesh arrays. Worker count leaves two cores free, so a view that meshes many cells never starves the rest of the system.

import type { LayerArrays } from "./instances";
import type { BiomeJson, BlockTableJson } from "./tables";
import type { WorkerRequest, WorkerResponse } from "./mesh-worker";

/** Raised when the mesh output changes, so meshes cached by an older build are not reused. */
export const MESH_VERSION = 1;

export interface MeshedCell {
  cell: [number, number, number];
  layers: LayerArrays[];
}

interface Job {
  id: number;
  bundle: ArrayBuffer;
  resolve(value: MeshedCell): void;
  reject(error: Error): void;
}

export function workerCount(): number {
  const cores = navigator.hardwareConcurrency || 2;
  return Math.max(1, Math.min(cores - 2, 6));
}

export class CellMesher {
  private readonly workers: Worker[] = [];
  private readonly idle: Worker[] = [];
  private readonly busy = new Map<Worker, Job>();
  private readonly queue: Job[] = [];
  private nextId = 1;
  private paused = false;

  constructor(blocks: BlockTableJson, biomes: BiomeJson[], count: number) {
    for (let index = 0; index < count; index++) {
      const worker = new Worker(new URL("./mesh-worker.ts", import.meta.url), { type: "module" });
      worker.onmessage = (event: MessageEvent<WorkerResponse>) => this.finish(worker, event.data);
      worker.onerror = (event) => this.fail(worker, new Error(event.message));
      worker.postMessage({ type: "tables", blocks, biomes } satisfies WorkerRequest);
      this.workers.push(worker);
      this.idle.push(worker);
    }
  }

  mesh(bundle: ArrayBuffer): Promise<MeshedCell> {
    return new Promise((resolve, reject) => {
      this.queue.push({ id: this.nextId++, bundle, resolve, reject });
      this.dispatch();
    });
  }

  dispose(): void {
    for (const job of [...this.queue, ...this.busy.values()]) {
      job.reject(new Error("the world map was closed"));
    }
    this.queue.length = 0;
    this.busy.clear();
    for (const worker of this.workers) {
      worker.terminate();
    }
  }

  pause(): void {
    this.paused = true;
  }

  resume(): void {
    this.paused = false;
    this.dispatch();
  }

  private dispatch(): void {
    while (!this.paused && this.idle.length > 0 && this.queue.length > 0) {
      const worker = this.idle.pop()!;
      const job = this.queue.shift()!;
      this.busy.set(worker, job);
      worker.postMessage({ type: "cell", id: job.id, bundle: job.bundle } satisfies WorkerRequest, [job.bundle]);
    }
  }

  private finish(worker: Worker, response: WorkerResponse): void {
    const job = this.busy.get(worker);
    this.busy.delete(worker);
    this.idle.push(worker);
    if (job && job.id === response.id) {
      if (response.type === "cell") {
        job.resolve({ cell: response.cell, layers: response.layers });
      } else {
        job.reject(new Error(response.message));
      }
    }
    this.dispatch();
  }

  private fail(worker: Worker, error: Error): void {
    const job = this.busy.get(worker);
    this.busy.delete(worker);
    this.idle.push(worker);
    job?.reject(error);
    this.dispatch();
  }
}
