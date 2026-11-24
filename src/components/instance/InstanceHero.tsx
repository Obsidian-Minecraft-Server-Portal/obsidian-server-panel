import { Icon } from "@iconify-icon/react";
import { formatDistanceToNow } from "date-fns";
import { useNavigate } from "react-router";
import { useInstalledContent } from "../../api/content";
import { hasPermission, type InstanceView } from "../../api/instances";
import { useSnapshots, type SnapshotInfo } from "../../api/backups";
import { formatBytes } from "../backups/format";
import { GhostButton } from "../ui/GhostButton";
import { InstanceBanner } from "./InstanceBanner";
import { InstanceIcon } from "./InstanceIcon";
import { ContentChip, LoaderChip, StatusChip, VersionChip } from "./InstanceChips";
import { PowerControls } from "./PowerControls";

export function InstanceBannerArea({ view }: { view: InstanceView }) {
  const navigate = useNavigate();
  return (
    <div className="relative h-[17rem] overflow-hidden">
      <div aria-hidden className="absolute inset-x-0 top-0 h-64">
        <InstanceBanner view={view} className="absolute inset-0" />
        <div className="grid-faint absolute inset-0 opacity-50" />
        <div className="absolute inset-0 bg-gradient-to-b from-transparent from-35% to-canvas" />
      </div>
      <div className="relative px-4 pt-6 sm:px-8">
        <GhostButton isIconOnly aria-label="Back to dashboard" onPress={() => navigate("/")} className="size-10">
          <Icon icon="lucide:chevron-left" width={18} height={18} />
        </GhostButton>
      </div>
    </div>
  );
}

export function InstanceIdentity({ view }: { view: InstanceView }) {
  const canReadFiles = hasPermission(view, "files.read");
  const canDownloadBackups = hasPermission(view, "backups.download");
  const content = useInstalledContent(view.id, canReadFiles);
  const snapshots = useSnapshots(view.id, canDownloadBackups);

  return (
    <div className="flex flex-wrap items-end gap-6 pb-2">
      <InstanceIcon instance={view} className="size-28 rounded-panel shadow-lift" />
      <div className="order-last w-full min-w-0 space-y-3 sm:order-none sm:w-auto sm:flex-1">
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
      <div className="ml-auto pb-1 sm:ml-0">
        <PowerControls view={view} />
      </div>
    </div>
  );
}

function backupSummary(snapshots: SnapshotInfo[] | undefined): string | undefined {
  if (!snapshots) return undefined;
  const latest = snapshots[0];
  if (!latest) return "no backups yet";
  return `${formatBytes(latest.size_bytes)} · last backup ${formatDistanceToNow(new Date(latest.created_at), { addSuffix: true })}`;
}
