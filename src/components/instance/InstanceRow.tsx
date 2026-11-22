import { Link } from "react-router";
import type { InstanceView } from "../../api/instances";
import { InstanceIcon } from "./InstanceIcon";
import { LoaderChip, StatusChip, VersionChip } from "./InstanceChips";
import { PowerControls } from "./PowerControls";

export function InstanceRow({ view }: { view: InstanceView }) {
  return (
    <div className="flex flex-wrap items-center gap-4 rounded-panel border border-line bg-panel px-4 py-3 transition-colors hover:border-line-strong">
      <InstanceIcon instance={view} className="size-10" />
      <Link to={`/instances/${view.id}`} className="min-w-40 flex-1 truncate font-semibold text-ink-0 hover:underline">
        {view.name}
      </Link>
      <div className="flex flex-wrap items-center gap-2">
        <StatusChip instance={view} />
        <LoaderChip instance={view} />
        <VersionChip mcVersion={view.mc_version} />
      </div>
      <span className="font-mono text-xs text-ink-3">:{view.port}</span>
      <PowerControls view={view} size="sm" />
    </div>
  );
}
