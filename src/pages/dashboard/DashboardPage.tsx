import { useQueries } from "@tanstack/react-query";
import { Icon } from "@iconify-icon/react";
import { useMemo, useState } from "react";
import { useNavigate } from "react-router";
import { instanceStatsOptions, useInstances, type InstanceStats, type InstanceView } from "../../api/instances";
import { useLiveInstances } from "../../api/instanceLive";
import { InstanceList } from "../../components/instance/InstanceList";
import { isRunning } from "../../components/instance/status";
import { LOADER_META } from "../../components/runtime/loaderMeta";
import { EmptyState } from "../../components/ui/EmptyState";
import { GhostButton } from "../../components/ui/GhostButton";
import { PageHeader } from "../../components/ui/PageHeader";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { ViewToggle } from "../../components/ui/ViewToggle";
import { StatTiles } from "./StatTiles";
import { useViewMode } from "./useViewMode";

function matches(view: InstanceView, query: string): boolean {
  if (!query) return true;
  const haystack = `${view.name} ${LOADER_META[view.loader].label} ${view.mc_version}`.toLowerCase();
  return haystack.includes(query);
}

export function DashboardPage() {
  const navigate = useNavigate();
  const instances = useInstances();
  const [search, setSearch] = useState("");
  const [mode, setMode] = useViewMode();
  const all = useMemo(() => instances.data ?? [], [instances.data]);
  const running = useMemo(() => all.filter((view) => isRunning(view.status)), [all]);
  const visible = useMemo(() => {
    const query = search.trim().toLowerCase();
    return all.filter((view) => matches(view, query));
  }, [all, search]);

  const ids = useMemo(() => all.map((view) => view.id), [all]);
  useLiveInstances(ids, { stats: true });
  const statsById = useQueries({
    queries: running.map((view) => instanceStatsOptions(view.id)),
    combine: (results) => new Map(running.map((view, index) => [view.id, results[index]?.data])),
  });

  const createInstance = () => navigate("/instances/new");
  const hasInstances = all.length > 0;

  return (
    <>
      <PageHeader
        title="Dashboard"
        subtitle="Your Minecraft servers and everything they're using."
        search={hasInstances ? { value: search, onChange: setSearch, placeholder: "Search instances" } : undefined}
        actions={
          <>
            {hasInstances && <ViewToggle label="Instance view" value={mode} onChange={setMode} />}
            <GhostButton onPress={() => navigate("/discover")}>Browse modpacks</GhostButton>
            <PrimaryButton onPress={createInstance}>
              <Icon icon="lucide:plus" width={16} height={16} />
              New instance
            </PrimaryButton>
          </>
        }
      />
      <div className="space-y-6 px-8 pb-8">
        {instances.isError && <p className="text-sm text-rose">{instances.error.message}</p>}
        {!instances.data && !instances.isError && <p className="text-sm text-ink-3">Loading instances…</p>}
        {instances.data && !hasInstances && (
          <EmptyState
            icon="lucide:box"
            title="No instances yet"
            description="Create a server from a Minecraft version, a loader, or a modpack."
            action={<PrimaryButton onPress={createInstance}>Create instance</PrimaryButton>}
          />
        )}
        {hasInstances && (
          <>
            <StatTiles totals={totalsOf(all, running, statsById)} />
            <section className="space-y-3">
              <h2 className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">
                Instances · {visible.length}
              </h2>
              {visible.length === 0 ? (
                <p className="py-16 text-center text-sm text-ink-3">No instances match “{search.trim()}”.</p>
              ) : (
                <InstanceList mode={mode} instances={visible} stats={statsById} />
              )}
            </section>
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
