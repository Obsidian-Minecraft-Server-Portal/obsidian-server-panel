import { Icon } from "@iconify-icon/react";
import { formatDistanceToNow } from "date-fns";
import { useNavigate } from "react-router";
import { useInstalledContent } from "../../api/content";
import { hasPermission, type InstanceView } from "../../api/instances";
import { useSnapshots, type SnapshotInfo } from "../../api/backups";
import { formatBytes } from "../backups/format";
import { GhostButton } from "../ui/GhostButton";
import { PixelBanner } from "../ui/PixelBanner";
import { InstanceIcon } from "./InstanceIcon";
import { ContentChip, LoaderChip, StatusChip, VersionChip } from "./InstanceChips";
import { PowerControls } from "./PowerControls";

export function InstanceHero({ view }: { view: InstanceView }) {
  const navigate = useNavigate();
  const canReadFiles = hasPermission(view, "files.read");
  const canDownloadBackups = hasPermission(view, "backups.download");
  const content = useInstalledContent(view.id, canReadFiles);
  const snapshots = useSnapshots(view.id, canDownloadBackups);

  return (
    <section className="relative overflow-hidden border-b border-line">
      <div aria-hidden className="absolute inset-x-0 top-0 h-64">
        <PixelBanner seed={String(view.id)} />
        <div className="grid-faint absolute inset-0 opacity-50" />
        <div className="absolute inset-0 bg-gradient-to-b from-transparent from-35% to-canvas" />
      </div>
      <div className="relative flex flex-col gap-6 px-8 pt-6">
        <div className="flex items-center gap-2">
          <GhostButton isIconOnly aria-label="Back to instances" onPress={() => navigate("/instances")} className="size-10">
            <Icon icon="lucide:chevron-left" width={18} height={18} />
          </GhostButton>
        </div>
        <div className="flex flex-wrap items-end gap-6 pt-16 pb-2">
          <InstanceIcon instance={view} className="size-28 rounded-panel shadow-glow" />
          <div className="min-w-0 flex-1 space-y-3">
            <div className="flex flex-wrap gap-2">
              <StatusChip instance={view} />
              <LoaderChip instance={view} />
              <VersionChip mcVersion={view.mc_version} />
              {content.data?.length ? <ContentChip items={content.data} /> : null}
            </div>
            <h1 className="truncate text-4xl font-semibold tracking-tight text-ink-0">{view.name}</h1>
            <p className="truncate font-mono text-xs text-ink-3" title={view.directory}>
              {[view.directory, canDownloadBackups && backupSummary(snapshots.data)].filter(Boolean).join(" · ")}
            </p>
          </div>
          <div className="pb-1">
            <PowerControls view={view} />
          </div>
        </div>
      </div>
    </section>
  );
}

function backupSummary(snapshots: SnapshotInfo[] | undefined): string | undefined {
  if (!snapshots) return undefined;
  const latest = snapshots[0];
  if (!latest) return "no backups yet";
  return `${formatBytes(latest.size_bytes)} · last backup ${formatDistanceToNow(new Date(latest.created_at), { addSuffix: true })}`;
}
