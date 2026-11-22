import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import { useNavigate } from "react-router";
import { instanceStatsOptions, useInstances, type InstanceStats, type InstanceView } from "../../api/instances";
import { useLiveInstances } from "../../api/instanceLive";
import { InstanceRow } from "../../components/instance/InstanceRow";
import { isRunning } from "../../components/instance/status";
import { EmptyState } from "../../components/ui/EmptyState";
import { PageHeader } from "../../components/ui/PageHeader";
import { Panel } from "../../components/ui/Panel";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { QuickActions } from "./QuickActions";
import { ResourceOverview } from "./ResourceOverview";
import { StatTiles } from "./StatTiles";

const RECENT_COUNT = 5;

export function DashboardPage() {
  const navigate = useNavigate();
  const instances = useInstances();
  const all = useMemo(() => instances.data ?? [], [instances.data]);
  const running = useMemo(() => all.filter((view) => isRunning(view.status)), [all]);
  const recent = useMemo(
    () => [...all].sort((a, b) => b.updated_at.localeCompare(a.updated_at)).slice(0, RECENT_COUNT),
    [all],
  );

  const ids = useMemo(() => all.map((view) => view.id), [all]);
  useLiveInstances(ids, { stats: true });
  const statsById = useQueries({
    queries: running.map((view) => instanceStatsOptions(view.id)),
    combine: (results) => new Map(running.map((view, index) => [view.id, results[index]?.data])),
  });

  return (
    <>
      <PageHeader title="Dashboard" subtitle="Everything running on this panel, at a glance." />
      <div className="space-y-6 px-8 pb-8">
        {instances.isError && <p className="text-sm text-rose">{instances.error.message}</p>}
        {instances.data && all.length === 0 ? (
          <EmptyState
            icon="lucide:layout-dashboard"
            title="No instances yet"
            description="Create your first Minecraft server to see its status, resources and activity here."
            action={<PrimaryButton onPress={() => navigate("/instances/new")}>Create instance</PrimaryButton>}
          />
        ) : (
          <>
            <StatTiles totals={totalsOf(all, running, statsById)} />
            <div className="grid gap-5 xl:grid-cols-[1fr_20rem]">
              <div className="space-y-5">
                <ResourceOverview running={running} statsById={statsById} />
                <Panel title="Recent instances">
                  <div className="flex flex-col gap-2">
                    {recent.map((view) => (
                      <InstanceRow key={view.id} view={view} />
                    ))}
                  </div>
                </Panel>
              </div>
              <QuickActions />
            </div>
          </>
        )}
      </div>
    </>
  );
}

function totalsOf(all: InstanceView[], running: InstanceView[], statsById: Map<number, InstanceStats | undefined>) {
  let players = 0;
  let cpuPercent = 0;
  let memoryBytes = 0;
  for (const view of running) {
    const stats = statsById.get(view.id);
    if (!stats) continue;
    players += stats.players_online ?? 0;
    cpuPercent += stats.cpu_percent;
    memoryBytes += stats.memory_bytes;
  }
  return { running: running.length, total: all.length, players, cpuPercent, memoryBytes };
}
