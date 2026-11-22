import { Link } from "react-router";
import type { InstanceStats, InstanceView } from "../../api/instances";
import { ProgressBar } from "../../components/ui/ProgressBar";
import { Panel } from "../../components/ui/Panel";
import { EmptyState } from "../../components/ui/EmptyState";
import { formatMemory } from "./format";

const MIB = 1024 * 1024;

interface ResourceOverviewProps {
  running: InstanceView[];
  statsById: Map<number, InstanceStats | undefined>;
}

export function ResourceOverview({ running, statsById }: ResourceOverviewProps) {
  return (
    <Panel title="Resources">
      {running.length === 0 ? (
        <EmptyState icon="lucide:activity" title="Nothing is running" description="Start a server to see its CPU, memory and players here." />
      ) : (
        <div className="space-y-5">
          {running.map((view) => {
            const stats = statsById.get(view.id);
            const memoryPercent = stats ? (stats.memory_bytes / (view.memory_max_mb * MIB)) * 100 : 0;
            return (
              <div key={view.id} className="space-y-2.5">
                <div className="flex flex-wrap items-baseline justify-between gap-2">
                  <Link to={`/instances/${view.id}`} className="font-semibold text-ink-0 hover:underline">
                    {view.name}
                  </Link>
                  <span className="font-mono text-xs text-ink-3">
                    {stats?.players_online ?? "—"}/{stats?.max_players ?? "—"} players
                  </span>
                </div>
                <Meter label="CPU" value={stats?.cpu_percent ?? 0} detail={`${(stats?.cpu_percent ?? 0).toFixed(0)}%`} />
                <Meter
                  label="Memory"
                  value={memoryPercent}
                  detail={`${formatMemory(stats?.memory_bytes ?? 0)} / ${formatMemory(view.memory_max_mb * MIB)}`}
                />
              </div>
            );
          })}
        </div>
      )}
    </Panel>
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
