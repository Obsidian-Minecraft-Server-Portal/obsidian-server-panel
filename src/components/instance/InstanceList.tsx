import type { InstanceStats, InstanceView } from "../../api/instances";
import type { ViewMode } from "../ui/ViewToggle";
import { InstanceCard } from "./InstanceCard";
import { InstanceRow } from "./InstanceRow";
import { InstanceTable } from "./InstanceTable";

interface InstanceListProps {
  mode: ViewMode;
  instances: InstanceView[];
  stats: Map<number, InstanceStats | undefined>;
}

export function InstanceList({ mode, instances, stats }: InstanceListProps) {
  if (mode === "table") return <InstanceTable instances={instances} stats={stats} />;
  if (mode === "compact") {
    return (
      <div className="divide-y divide-line overflow-hidden rounded-panel border border-line bg-panel">
        {instances.map((view) => (
          <InstanceRow key={view.id} view={view} stats={stats.get(view.id)} />
        ))}
      </div>
    );
  }
  return (
    <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
      {instances.map((view) => (
        <InstanceCard key={view.id} view={view} stats={stats.get(view.id)} />
      ))}
    </div>
  );
}
