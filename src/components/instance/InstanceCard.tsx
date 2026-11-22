import { Link, useNavigate } from "react-router";
import type { InstanceView } from "../../api/instances";
import { GlassCard } from "../ui/GlassCard";
import { InstanceBanner } from "./InstanceBanner";
import { InstanceIcon } from "./InstanceIcon";
import { LoaderChip, StatusChip, VersionChip } from "./InstanceChips";
import { PowerControls } from "./PowerControls";

export function InstanceCard({ view }: { view: InstanceView }) {
  const navigate = useNavigate();
  const path = `/instances/${view.id}`;

  return (
    <GlassCard
      onClick={(event) => {
        if (!event.defaultPrevented) navigate(path);
      }}
      className="group flex cursor-pointer flex-col transition-colors duration-150 hover:border-line-strong"
    >
      <div className="relative h-24 overflow-hidden">
        <InstanceBanner view={view} className="absolute inset-0" />
        <div className="absolute inset-0 bg-gradient-to-b from-transparent to-panel" />
      </div>
      <div className="relative z-10 flex flex-1 flex-col gap-4 p-5 pt-0">
        <div className="-mt-7 flex items-end gap-3">
          <InstanceIcon instance={view} className="size-14 border-2 border-panel shadow-lift" />
          <div className="min-w-0 flex-1 pb-1">
            <Link to={path} className="block truncate text-lg font-semibold text-ink-0 hover:underline">
              {view.name}
            </Link>
            <p className="truncate font-mono text-[11px] text-ink-3">:{view.port}</p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <StatusChip instance={view} />
          <LoaderChip instance={view} />
          <VersionChip mcVersion={view.mc_version} />
        </div>
        <div className="mt-auto pt-1" onClick={(event) => event.stopPropagation()}>
          <PowerControls view={view} size="sm" />
        </div>
      </div>
    </GlassCard>
  );
}
