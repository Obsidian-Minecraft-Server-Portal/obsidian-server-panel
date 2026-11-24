import { memo } from "react";
import { Link, useNavigate } from "react-router";
import type { InstanceStats, InstanceView } from "../../api/instances";
import { ContextMenu, MenuButton } from "../ui/ContextMenu";
import { GlassCard } from "../ui/GlassCard";
import { useContextMenu } from "../ui/useContextMenu";
import { InstanceBanner } from "./InstanceBanner";
import { InstanceIcon } from "./InstanceIcon";
import { LoaderChip, StatusChip, VersionChip } from "./InstanceChips";
import { ResourceMeters } from "./InstanceResources";
import { PowerControls } from "./PowerControls";
import { useInstanceMenu } from "./useInstanceMenu";

interface InstanceCardProps {
  view: InstanceView;
  stats: InstanceStats | undefined;
}

export const InstanceCard = memo(function InstanceCard({ view, stats }: InstanceCardProps) {
  const navigate = useNavigate();
  const path = `/instances/${view.id}`;
  const actions = useInstanceMenu(view);
  const menu = useContextMenu();

  return (
    <GlassCard
      onClick={(event) => {
        if (!event.defaultPrevented) navigate(path);
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        menu.openAt(event.clientX, event.clientY);
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
            <Link to={path} className="block truncate text-lg font-semibold text-ink-0">
              {view.name}
            </Link>
            <p className="truncate font-mono text-[11px] text-ink-3">:{view.port}</p>
          </div>
          <MenuButton label={`${view.name} actions`} onOpen={menu.openAt} />
        </div>
        <div className="flex flex-wrap gap-2">
          <StatusChip instance={view} />
          <LoaderChip instance={view} />
          <VersionChip mcVersion={view.mc_version} />
        </div>
        <div className="mt-auto space-y-4 pt-1" onClick={(event) => event.stopPropagation()}>
          <PowerControls view={view} size="sm" />
          <ResourceMeters view={view} stats={stats} />
        </div>
      </div>
      {menu.menu && (
        <ContextMenu
          x={menu.menu.x}
          y={menu.menu.y}
          label={`${view.name} actions`}
          actions={actions}
          onClose={menu.close}
        />
      )}
    </GlassCard>
  );
});
