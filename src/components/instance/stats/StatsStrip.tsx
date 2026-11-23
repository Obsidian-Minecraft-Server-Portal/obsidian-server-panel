import { cn } from "@heroui/react";
import type { ReactNode } from "react";
import type { InstanceStats } from "../../../api/instances";
import { GlassCard } from "../../ui/GlassCard";
import { Sparkline } from "./Sparkline";

const MIB = 1024 * 1024;
const CPU_BASE_MAX = 100;

interface StatsStripProps {
  running: boolean;
  stats: InstanceStats | undefined;
  cpu: number[];
  memory: number[];
  memoryMaxMb: number;
}

export function StatsStrip({ running, stats, cpu, memory, memoryMaxMb }: StatsStripProps) {
  if (!running) {
    return (
      <div className="rounded-panel border border-dashed border-line-strong px-5 py-4 font-mono text-[11px] tracking-wider text-ink-3 uppercase">
        Server offline
      </div>
    );
  }

  const cpuMax = Math.max(CPU_BASE_MAX, ...cpu);

  return (
    <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
      <Tile label="CPU">
        <Value>{stats ? `${stats.cpu_percent.toFixed(0)}%` : "—"}</Value>
        <Sparkline values={cpu} max={cpuMax} className="h-8 w-full overflow-visible" />
      </Tile>
      <Tile label="Memory">
        <Value>{stats ? `${Math.round(stats.memory_bytes / MIB)} / ${memoryMaxMb} MB` : "—"}</Value>
        <Sparkline values={memory} max={Math.max(memoryMaxMb * MIB, ...memory)} className="h-8 w-full overflow-visible" />
      </Tile>
      <Tile label="Players">
        <Value>{`${stats?.players_online ?? "—"}/${stats?.max_players ?? "—"}`}</Value>
        <p className="truncate font-mono text-[11px] text-ink-3" title={stats?.motd ?? undefined}>
          {stats?.motd ?? "\u00a0"}
        </p>
      </Tile>
      <Tile label="TPS">
        <Value className={stats?.tps != null ? tpsClass(stats.tps) : "text-ink-3"}>
          {stats?.tps != null ? stats.tps.toFixed(1) : "—"}
        </Value>
      </Tile>
    </div>
  );
}

function tpsClass(tps: number): string {
  if (tps >= 19) return "text-green";
  if (tps >= 15) return "text-amber";
  return "text-rose";
}

function Tile({ label, children }: { label: string; children: ReactNode }) {
  return (
    <GlassCard className="animate-tick-in p-4">
      <div className="space-y-2">
        <span className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">{label}</span>
        {children}
      </div>
    </GlassCard>
  );
}

function Value({ children, className = "text-ink-0" }: { children: ReactNode; className?: string }) {
  return <p className={cn("font-mono text-xl font-semibold tabular-nums", className)}>{children}</p>;
}
