import { Link } from "react-router";
import type { InstanceStats, InstanceView } from "../../api/instances";
import { ContextMenu, MenuButton } from "../ui/ContextMenu";
import { useContextMenu } from "../ui/useContextMenu";
import { InstanceIcon } from "./InstanceIcon";
import { LoaderChip, StatusChip, VersionChip } from "./InstanceChips";
import { ResourceReadouts } from "./InstanceResources";
import { PowerControls } from "./PowerControls";
import { useInstanceMenu } from "./useInstanceMenu";

interface InstanceRowProps {
  view: InstanceView;
  stats: InstanceStats | undefined;
}

export function InstanceRow({ view, stats }: InstanceRowProps) {
  const actions = useInstanceMenu(view);
  const menu = useContextMenu();

  return (
    <div
      onContextMenu={(event) => {
        event.preventDefault();
        menu.openAt(event.clientX, event.clientY);
      }}
      className="flex flex-wrap items-center gap-4 px-4 py-3 transition-colors hover:bg-raised/40"
    >
      <InstanceIcon instance={view} className="size-10" />
      <Link to={`/instances/${view.id}`} className="min-w-40 flex-1 truncate font-semibold text-ink-0 hover:underline">
        {view.name}
      </Link>
      <div className="flex flex-wrap items-center gap-2">
        <StatusChip instance={view} />
        <LoaderChip instance={view} />
        <VersionChip mcVersion={view.mc_version} />
      </div>
      <ResourceReadouts view={view} stats={stats} />
      <span className="font-mono text-xs text-ink-3">:{view.port}</span>
      <PowerControls view={view} size="sm" />
      <MenuButton label={`${view.name} actions`} onOpen={menu.openAt} />
      {menu.menu && (
        <ContextMenu
          x={menu.menu.x}
          y={menu.menu.y}
          label={`${view.name} actions`}
          actions={actions}
          onClose={menu.close}
        />
      )}
    </div>
  );
}
