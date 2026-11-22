import { format } from "date-fns";
import { Link } from "react-router";
import type { InstanceView } from "../../api/instances";
import { LOADER_META } from "../runtime/loaderMeta";
import { StatusChip } from "./InstanceChips";
import { PowerControls } from "./PowerControls";

const HEAD = ["Name", "Status", "Server type", "Version", "Port", "Memory", "Created", ""];

export function InstanceTable({ instances }: { instances: InstanceView[] }) {
  return (
    <div className="overflow-x-auto rounded-panel border border-line bg-panel">
      <table className="w-full min-w-[760px] border-collapse text-left text-sm">
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
            <tr key={view.id} className="border-b border-line last:border-b-0 hover:bg-raised/40">
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
              <td className="px-4 py-3 font-mono text-xs text-ink-2">
                {view.memory_min_mb}–{view.memory_max_mb} MB
              </td>
              <td className="px-4 py-3 font-mono text-xs text-ink-3">{format(new Date(view.created_at), "d MMM yyyy")}</td>
              <td className="px-4 py-3">
                <div className="flex justify-end">
                  <PowerControls view={view} size="sm" />
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
