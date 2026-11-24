import { Modal } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import {
  type InstalledContent,
  type PlatformId,
  useApplyContentUpdate,
  useProjectVersions,
} from "../../api/content";
import type { InstanceView } from "../../api/instances";
import { CheckboxRow } from "../ui/CheckboxRow";
import { EmptyState } from "../ui/EmptyState";
import { GhostButton } from "../ui/GhostButton";
import { SectionTabs } from "../ui/SectionTabs";
import { VersionRow } from "./VersionRow";
import { CHANNEL_TABS, versionFilter, type VersionChannel } from "./versionFilters";

const PAGE_SIZE = 20;
const DISABLED_SUFFIX = /\.disabled$/;

interface SwitchVersionModalProps {
  instance: InstanceView | null;
  item: InstalledContent | null;
  onClose: () => void;
}

export function SwitchVersionModal({ instance, item, onClose }: SwitchVersionModalProps) {
  const linked = item && item.platform !== null && item.project_id !== null ? { item, platform: item.platform, projectId: item.project_id } : null;
  return (
    <Modal.Backdrop isOpen={linked !== null && instance !== null} onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="lg">
        <Modal.Dialog className="bg-panel text-ink-1">
          {linked && instance && (
            <SwitchVersionBody
              key={linked.item.id}
              instance={instance}
              item={linked.item}
              platform={linked.platform}
              projectId={linked.projectId}
              onClose={onClose}
            />
          )}
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

interface SwitchVersionBodyProps {
  instance: InstanceView;
  item: InstalledContent;
  platform: PlatformId;
  projectId: string;
  onClose: () => void;
}

function SwitchVersionBody({ instance, item, platform, projectId, onClose }: SwitchVersionBodyProps) {
  const [channel, setChannel] = useState<VersionChannel>("all");
  const [matchInstance, setMatchInstance] = useState(true);
  const [limit, setLimit] = useState(PAGE_SIZE);
  const versions = useProjectVersions(platform, projectId, versionFilter(item.kind, matchInstance ? instance : null, null));
  const applyUpdate = useApplyContentUpdate(instance.id);
  const title = item.name ?? item.filename.replace(DISABLED_SUFFIX, "");
  const listed = (versions.data ?? []).filter((version) => channel === "all" || version.channel === channel);
  const shown = listed.slice(0, limit);

  return (
    <>
      <Modal.Header className="flex items-start gap-3">
        <span className="grid size-10 shrink-0 place-items-center rounded-control bg-secondary-tint text-secondary">
          <Icon icon="lucide:history" width={18} height={18} />
        </span>
        <div className="min-w-0 space-y-1">
          <Modal.Heading className="truncate text-lg font-semibold text-ink-0">Switch version · {title}</Modal.Heading>
          <p className="truncate font-mono text-[11px] text-ink-3">{item.filename.replace(DISABLED_SUFFIX, "")}</p>
        </div>
      </Modal.Header>
      <Modal.Body className="space-y-4">
        <div className="flex flex-wrap items-end gap-4">
          <SectionTabs
            label="Release channel"
            tabs={CHANNEL_TABS}
            value={channel}
            onChange={setChannel}
            className="max-w-full overflow-x-auto"
          />
          <CheckboxRow isSelected={matchInstance} onChange={setMatchInstance}>
            {`Only versions that match ${instance.name}`}
          </CheckboxRow>
        </div>
        {versions.isPending && <p className="text-sm text-ink-3">Loading versions…</p>}
        {versions.isError && <p className="text-sm text-rose">{versions.error.message}</p>}
        {applyUpdate.error && <p className="text-sm text-rose">{applyUpdate.error.message}</p>}
        {versions.isSuccess && listed.length === 0 && (
          <EmptyState icon="lucide:list-x" title="No versions" description="Nothing matches these filters." />
        )}
        <div className="space-y-2.5">
          {shown.map((version) => {
            const installed = version.id === item.version_id;
            const switching = applyUpdate.isPending && applyUpdate.variables?.versionId === version.id;
            return (
              <VersionRow
                key={version.id}
                version={version}
                actionLabel={installed ? "Installed" : switching ? "Switching…" : "Switch"}
                actionDisabled={installed || applyUpdate.isPending}
                matchesFile={version.file_name === item.filename.replace(DISABLED_SUFFIX, "")}
                onAction={() => applyUpdate.mutate({ modId: item.id, versionId: version.id }, { onSuccess: onClose })}
              />
            );
          })}
        </div>
        {listed.length > shown.length && (
          <div className="flex justify-center">
            <GhostButton onPress={() => setLimit((count) => count + PAGE_SIZE)}>Show more</GhostButton>
          </div>
        )}
      </Modal.Body>
    </>
  );
}
