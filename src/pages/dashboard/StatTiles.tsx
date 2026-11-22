import type { ReactNode } from "react";
import { GlassCard } from "../../components/ui/GlassCard";
import { formatMemory } from "./format";

interface Totals {
  running: number;
  total: number;
  players: number;
  cpuPercent: number;
  memoryBytes: number;
}

export function StatTiles({ totals }: { totals: Totals }) {
  return (
    <div className="grid gap-5 sm:grid-cols-2 xl:grid-cols-4">
      <Tile label="Servers running" value={`${totals.running} / ${totals.total}`} />
      <Tile label="Players online" value={String(totals.players)} />
      <Tile label="CPU" value={`${totals.cpuPercent.toFixed(0)}%`} hint="Summed across running servers" />
      <Tile label="Memory" value={formatMemory(totals.memoryBytes)} hint="Resident set of running servers" />
    </div>
  );
}

function Tile({ label, value, hint }: { label: string; value: string; hint?: ReactNode }) {
  return (
    <GlassCard className="p-5">
      <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">{label}</p>
      <p className="mt-2 text-3xl font-semibold tracking-tight text-ink-0">{value}</p>
      {hint && <p className="mt-1 text-xs text-ink-4">{hint}</p>}
    </GlassCard>
  );
}
