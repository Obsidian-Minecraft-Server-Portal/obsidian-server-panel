export interface MapStatsValues {
  fps: number;
  frameMs: number;
  scriptMs: number;
  gpu: string;
  drawCalls: number;
  triangles: number;
  textures: number;
  geometries: number;
  heapMb: number | null;
  cellsLoaded: number;
  cellsQueued: number;
  cellsLoading: number;
  cellsFailed: number;
  workersBusy: number;
  workersQueued: number;
}

const PUBLISH_MS = 500;

/**
 * The 3D view's numbers. The view writes them every frame, and subscribers get a copy twice a second, so React renders at that
 * pace rather than once per frame.
 */
export class MapStats {
  private pending: Partial<MapStatsValues> = {};
  private published: Partial<MapStatsValues> = {};
  private readonly listeners = new Set<() => void>();
  private timer = 0;

  set(values: Partial<MapStatsValues>): void {
    Object.assign(this.pending, values);
  }

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    if (this.timer === 0) {
      this.timer = window.setInterval(() => this.publish(), PUBLISH_MS);
    }
    return () => {
      this.listeners.delete(listener);
      if (this.listeners.size === 0) {
        window.clearInterval(this.timer);
        this.timer = 0;
      }
    };
  };

  snapshot = (): Partial<MapStatsValues> => this.published;

  private publish(): void {
    this.published = { ...this.pending };
    for (const listener of this.listeners) listener();
  }
}
