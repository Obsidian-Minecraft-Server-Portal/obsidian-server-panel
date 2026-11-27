import { useSyncExternalStore } from "react";
import { useLiveInstances } from "../../api/instanceLive";
import { useInstanceStats } from "../../api/instances";
import type { MapStats, MapStatsValues } from "./map-stats";

/**
 * The 3D view's frame rate and load, with the server's CPU and memory. A page cannot read the browser's own CPU use, so the
 * script time per frame stands in for it.
 */
export function MapStatsCard({ instanceId, stats, live }: { instanceId: number; stats: MapStats; live: boolean }) {
  const values = useSyncExternalStore(stats.subscribe, stats.snapshot);
  useLiveInstances([instanceId], { status: false, stats: true });
  const server = useInstanceStats(instanceId).data;
  return (
    <div className="w-80 max-w-[calc(100%-1.5rem)] rounded-control border border-line-strong bg-canvas/85 px-3 py-2.5 font-mono text-[11px] text-ink-2 shadow-lift backdrop-blur-md">
      {!live && <p className="mb-2 text-ink-3">Shown in the 3D view.</p>}
      <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
        <Row label="FPS" value={fixed(values.fps, 0)} />
        <Row label="Frame" value={fixed(values.frameMs, 1, " ms")} />
        <Row label="Script per frame" value={fixed(values.scriptMs, 2, " ms")} />
        <Row label="GPU" value={values.gpu ?? "–"} />
        <Row label="Draw calls" value={fixed(values.drawCalls, 0)} />
        <Row label="Triangles" value={thousands(values.triangles)} />
        <Row label="Textures" value={fixed(values.textures, 0)} />
        <Row label="Geometries" value={fixed(values.geometries, 0)} />
        <Row label="JS heap" value={values.heapMb === null || values.heapMb === undefined ? "n/a" : fixed(values.heapMb, 0, " MB")} />
        <Row label="Cells" value={cells(values)} />
        <Row label="Mesh workers" value={`${values.workersBusy ?? 0} busy · ${values.workersQueued ?? 0} waiting`} />
        <Row label="Server CPU" value={server ? fixed(server.cpu_percent, 0, "%") : "n/a"} />
        <Row label="Server RAM" value={server ? fixed(server.memory_bytes / 1048576, 0, " MB") : "n/a"} />
      </dl>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <>
      <dt className="text-ink-3">{label}</dt>
      <dd className="truncate text-right text-ink-1">{value}</dd>
    </>
  );
}

function fixed(value: number | undefined, digits: number, unit = ""): string {
  return value === undefined || !Number.isFinite(value) ? "–" : `${value.toFixed(digits)}${unit}`;
}

function thousands(value: number | undefined): string {
  return value === undefined ? "–" : value >= 1000 ? `${(value / 1000).toFixed(1)}k` : `${value}`;
}

function cells(values: Partial<MapStatsValues>): string {
  return `${values.cellsLoaded ?? 0} loaded · ${values.cellsLoading ?? 0} loading · ${values.cellsQueued ?? 0} queued`;
}
