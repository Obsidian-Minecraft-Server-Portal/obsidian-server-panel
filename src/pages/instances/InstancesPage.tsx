import { Icon } from "@iconify-icon/react";
import { useMemo, useState } from "react";
import { useNavigate } from "react-router";
import { useInstances, type InstanceView } from "../../api/instances";
import { useLiveInstances } from "../../api/instanceLive";
import { EmptyState } from "../../components/ui/EmptyState";
import { GhostButton } from "../../components/ui/GhostButton";
import { PageHeader } from "../../components/ui/PageHeader";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { InstanceCard } from "../../components/instance/InstanceCard";
import { InstanceRow } from "../../components/instance/InstanceRow";
import { InstanceTable } from "../../components/instance/InstanceTable";
import { LOADER_META } from "../../components/runtime/loaderMeta";
import { ViewModeSwitch } from "./ViewModeSwitch";
import { useViewMode, type ViewMode } from "./useViewMode";

function matches(view: InstanceView, query: string): boolean {
  if (!query) return true;
  const haystack = `${view.name} ${LOADER_META[view.loader].label} ${view.mc_version}`.toLowerCase();
  return haystack.includes(query);
}

export function InstancesPage() {
  const navigate = useNavigate();
  const instances = useInstances();
  const [search, setSearch] = useState("");
  const [mode, setMode] = useViewMode();
  const all = instances.data;

  const ids = useMemo(() => all?.map((view) => view.id) ?? [], [all]);
  useLiveInstances(ids);

  const visible = useMemo(() => {
    const query = search.trim().toLowerCase();
    return all?.filter((view) => matches(view, query)) ?? [];
  }, [all, search]);

  const hasInstances = (all?.length ?? 0) > 0;

  return (
    <>
      <PageHeader
        title="Instances"
        subtitle="Your Minecraft servers, running and stopped."
        search={{ value: search, onChange: setSearch, placeholder: "Search instances" }}
        actions={
          <>
            <ViewModeSwitch value={mode} onChange={setMode} />
            <GhostButton onPress={() => navigate("/discover")}>Browse modpacks</GhostButton>
            <PrimaryButton onPress={() => navigate("/instances/new")}>
              <Icon icon="lucide:plus" width={16} height={16} />
              New instance
            </PrimaryButton>
          </>
        }
      />
      <div className="px-8 pb-8">
        {instances.isError && <p className="text-sm text-rose">{instances.error.message}</p>}
        {!all && !instances.isError && <p className="text-sm text-ink-3">Loading instances…</p>}
        {all && !hasInstances && (
          <EmptyState
            icon="lucide:box"
            title="No instances"
            description="Create a server from a Minecraft version, a loader, or a modpack."
            action={<PrimaryButton onPress={() => navigate("/instances/new")}>Create instance</PrimaryButton>}
          />
        )}
        {all && hasInstances && visible.length === 0 && (
          <p className="py-16 text-center text-sm text-ink-3">No instances match “{search.trim()}”.</p>
        )}
        {visible.length > 0 && <InstanceList mode={mode} instances={visible} />}
      </div>
    </>
  );
}

function InstanceList({ mode, instances }: { mode: ViewMode; instances: InstanceView[] }) {
  if (mode === "table") return <InstanceTable instances={instances} />;
  if (mode === "compact") {
    return (
      <div className="flex flex-col gap-2">
        {instances.map((view) => (
          <InstanceRow key={view.id} view={view} />
        ))}
      </div>
    );
  }
  return (
    <div className="grid gap-5 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
      {instances.map((view) => (
        <InstanceCard key={view.id} view={view} />
      ))}
    </div>
  );
}
