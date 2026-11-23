import { format } from "date-fns";
import { Link } from "react-router";
import type { InstanceStats, InstanceView } from "../../api/instances";
import { ContextMenu, MenuButton } from "../ui/ContextMenu";
import { useContextMenu } from "../ui/useContextMenu";
import { LOADER_META } from "../runtime/loaderMeta";
import { StatusChip } from "./InstanceChips";
import { readoutOf } from "./InstanceResources";
import { PowerControls } from "./PowerControls";
import { useInstanceMenu } from "./useInstanceMenu";

const HEAD = ["Name", "Status", "Server type", "Version", "Port", "CPU", "Players", "Memory", "Created", ""];

interface InstanceTableProps {
  instances: InstanceView[];
  stats: Map<number, InstanceStats | undefined>;
}

export function InstanceTable({ instances, stats }: InstanceTableProps) {
  return (
    <div className="overflow-x-auto rounded-panel border border-line bg-panel">
      <table className="w-full min-w-[1040px] border-collapse text-left text-sm">
        <thead>
          <tr className="border-b border-line">
            {HEAD.map((label) => (
              <th key={label} className="px-4 py-3 font-mono text-[11px] font-medium tracking-wider text-ink-3 uppercase">
                {label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {instances.map((view) => (
            <InstanceTableRow key={view.id} view={view} stats={stats.get(view.id)} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

interface InstanceTableRowProps {
  view: InstanceView;
  stats: InstanceStats | undefined;
}

function InstanceTableRow({ view, stats }: InstanceTableRowProps) {
  const actions = useInstanceMenu(view);
  const menu = useContextMenu();
  const readout = readoutOf(view, stats);

  return (
    <tr
      onContextMenu={(event) => {
        event.preventDefault();
        menu.openAt(event.clientX, event.clientY);
      }}
      className="border-b border-line last:border-b-0 hover:bg-raised/40"
    >
      <td className="px-4 py-3">
        <Link to={`/instances/${view.id}`} className="font-semibold text-ink-0 hover:underline">
          {view.name}
        </Link>
      </td>
      <td className="px-4 py-3">
        <StatusChip instance={view} />
      </td>
      <td className="px-4 py-3 text-ink-2">{LOADER_META[view.loader].label}</td>
      <td className="px-4 py-3 font-mono text-xs text-ink-2">{view.mc_version}</td>
      <td className="px-4 py-3 font-mono text-xs text-ink-2">{view.port}</td>
      <td className="px-4 py-3 font-mono text-xs text-ink-2">{readout.cpu}</td>
      <td className="px-4 py-3 font-mono text-xs text-ink-2">{readout.players}</td>
      <td className="px-4 py-3 font-mono text-xs whitespace-nowrap text-ink-2">{readout.memory}</td>
      <td className="px-4 py-3 font-mono text-xs text-ink-3">{format(new Date(view.created_at), "d MMM yyyy")}</td>
      <td className="px-4 py-3">
        <div className="flex items-center justify-end gap-2">
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
      </td>
    </tr>
  );
}
