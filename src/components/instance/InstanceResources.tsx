import type { InstanceStats, InstanceView } from "../../api/instances";
import { ProgressBar } from "../ui/ProgressBar";
import { isRunning } from "./status";

const MIB = 1024 * 1024;
const GIB = 1024 * MIB;
const EMPTY = "—";

interface Readout {
  cpu: string;
  players: string;
  memory: string;
}

interface LiveUsage {
  cpuPercent: number;
  playersPercent: number;
  memoryPercent: number;
  readout: Readout;
}

interface ResourceProps {
  view: InstanceView;
  stats: InstanceStats | undefined;
}

const OFFLINE: Readout = { cpu: EMPTY, players: EMPTY, memory: EMPTY };

function formatGb(bytes: number): string {
  return String(Number((bytes / GIB).toFixed(1)));
}

function liveUsage(view: InstanceView, stats: InstanceStats | undefined): LiveUsage | null {
  if (!stats || !isRunning(view.status)) return null;
  const totalBytes = view.memory_max_mb * MIB;
  const online = stats.players_online ?? null;
  const max = stats.max_players ?? null;
  return {
    cpuPercent: stats.cpu_percent,
    playersPercent: online !== null && max ? (online / max) * 100 : 0,
    memoryPercent: totalBytes > 0 ? (stats.memory_bytes / totalBytes) * 100 : 0,
    readout: {
      cpu: `${stats.cpu_percent.toFixed(0)}%`,
      players: `${online ?? EMPTY}/${max ?? EMPTY}`,
      memory: `${formatGb(stats.memory_bytes)} / ${formatGb(totalBytes)} GB`,
    },
  };
}

export function readoutOf(view: InstanceView, stats: InstanceStats | undefined): Readout {
  return liveUsage(view, stats)?.readout ?? OFFLINE;
}

export function ResourceMeters({ view, stats }: ResourceProps) {
  const usage = liveUsage(view, stats);
  if (!usage) return <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">Offline</p>;
  return (
    <div className="space-y-2.5">
      <Meter label="CPU" value={usage.cpuPercent} detail={usage.readout.cpu} />
      <Meter label="Players" value={usage.playersPercent} detail={usage.readout.players} />
      <Meter label="Memory" value={usage.memoryPercent} detail={usage.readout.memory} />
    </div>
  );
}

export function ResourceReadouts({ view, stats }: ResourceProps) {
  const readout = readoutOf(view, stats);
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[11px] text-ink-2">
      <span>CPU: {readout.cpu}</span>
      <span>Players: {readout.players}</span>
      <span>Memory: {readout.memory}</span>
    </div>
  );
}

function Meter({ label, value, detail }: { label: string; value: number; detail: string }) {
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between font-mono text-[11px] text-ink-3">
        <span className="tracking-wider uppercase">{label}</span>
        <span>{detail}</span>
      </div>
      <ProgressBar value={value} />
    </div>
  );
}
