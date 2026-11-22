import { Link } from "react-router";
import type { InstanceView } from "../../api/instances";
import { GlassCard } from "../ui/GlassCard";
import { PixelBanner } from "../ui/PixelBanner";
import { InstanceIcon } from "./InstanceIcon";
import { LoaderChip, StatusChip, VersionChip } from "./InstanceChips";
import { PowerControls } from "./PowerControls";

export function InstanceCard({ view }: { view: InstanceView }) {
  return (
    <GlassCard className="flex flex-col">
      <div className="relative h-24 overflow-hidden">
        <PixelBanner seed={String(view.id)} />
        <div className="absolute inset-0 bg-gradient-to-b from-transparent to-panel" />
      </div>
      <div className="flex flex-1 flex-col gap-4 p-5 pt-0">
        <div className="-mt-7 flex items-end gap-3">
          <InstanceIcon instance={view} className="size-14 shadow-glow" />
          <div className="min-w-0 flex-1 pb-1">
            <Link to={`/instances/${view.id}`} className="block truncate text-lg font-semibold text-ink-0 hover:text-accent">
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
        <div className="mt-auto pt-1">
          <PowerControls view={view} size="sm" />
        </div>
      </div>
    </GlassCard>
  );
}
