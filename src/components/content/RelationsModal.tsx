import { Modal } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { type DependencyEntry, type DependentEntry, type InstalledContent, useContentRelations } from "../../api/content";
import { Chip } from "../ui/Chip";
import { EmptyState } from "../ui/EmptyState";
import { SectionTabs, type SectionTab } from "../ui/SectionTabs";

export type RelationsTab = "dependencies" | "dependents";

export interface RelationsTarget {
  item: InstalledContent;
  tab: RelationsTab;
}

interface RelationsModalProps {
  instanceId: number;
  target: RelationsTarget | null;
  onClose: () => void;
}

export function RelationsModal({ instanceId, target, onClose }: RelationsModalProps) {
  return (
    <Modal.Backdrop isOpen={target !== null} onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="md">
        <Modal.Dialog className="bg-panel text-ink-1">
          {target && <RelationsBody key={target.item.id} instanceId={instanceId} target={target} />}
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

function RelationsBody({ instanceId, target }: { instanceId: number; target: RelationsTarget }) {
  const { item } = target;
  const [tab, setTab] = useState<RelationsTab>(target.tab);
  const relations = useContentRelations(instanceId, item.id);
  const tabs: SectionTab<RelationsTab>[] = [
    { id: "dependencies", label: `Dependencies${relations.data ? ` · ${relations.data.dependencies.length}` : ""}` },
    { id: "dependents", label: `Dependents${relations.data ? ` · ${relations.data.dependents.length}` : ""}` },
  ];

  return (
    <>
      <Modal.Header className="flex items-start gap-3">
        <span className="grid size-10 shrink-0 place-items-center rounded-control bg-secondary-tint text-secondary">
          <Icon icon="lucide:network" width={18} height={18} />
        </span>
        <div className="min-w-0 space-y-1">
          <Modal.Heading className="truncate text-lg font-semibold text-ink-0">{item.name ?? item.filename}</Modal.Heading>
          <p className="truncate font-mono text-[11px] text-ink-3">{item.filename}</p>
        </div>
      </Modal.Header>
      <Modal.Body className="space-y-4">
        <SectionTabs label="Relations" tabs={tabs} value={tab} onChange={setTab} />
        {relations.isPending && <p className="font-mono text-xs text-ink-3">Reading mod metadata…</p>}
        {relations.isError && <p className="text-sm text-rose">{relations.error.message}</p>}
        {relations.data &&
          (tab === "dependencies" ? (
            <DependencyList entries={relations.data.dependencies} />
          ) : (
            <DependentList entries={relations.data.dependents} />
          ))}
      </Modal.Body>
    </>
  );
}

function DependencyList({ entries }: { entries: DependencyEntry[] }) {
  if (entries.length === 0) {
    return (
      <EmptyState
        icon="lucide:package-check"
        title="No dependencies"
        description="This file does not declare any mod or plugin it needs."
      />
    );
  }
  return (
    <ul className="space-y-2">
      {entries.map((entry) => (
        <li
          key={entry.id}
          className="flex flex-wrap items-center justify-between gap-3 rounded-control border border-line bg-input/60 px-3 py-2.5"
        >
          <div className="min-w-0 flex-1">
            <p className="truncate font-medium text-ink-0">{entry.installed?.name ?? entry.installed?.filename ?? entry.id}</p>
            <p className="truncate font-mono text-[11px] text-ink-3">{entry.id}</p>
          </div>
          <div className="flex shrink-0 flex-wrap gap-1.5">
            <Chip tone={entry.required ? "neutral" : "secondary"}>{entry.required ? "Required" : "Optional"}</Chip>
            {entry.installed ? (
              <Chip tone="green">Installed</Chip>
            ) : (
              <Chip tone={entry.required ? "amber" : "neutral"}>Missing</Chip>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}

function DependentList({ entries }: { entries: DependentEntry[] }) {
  if (entries.length === 0) {
    return (
      <EmptyState
        icon="lucide:package-search"
        title="Nothing depends on this"
        description="No other installed mod or plugin declares it as a dependency."
      />
    );
  }
  return (
    <ul className="space-y-2">
      {entries.map((entry) => (
        <li
          key={entry.installed.id}
          className="flex flex-wrap items-center justify-between gap-3 rounded-control border border-line bg-input/60 px-3 py-2.5"
        >
          <div className="min-w-0 flex-1">
            <p className="truncate font-medium text-ink-0">{entry.installed.name ?? entry.installed.filename}</p>
            <p className="truncate font-mono text-[11px] text-ink-3">{entry.installed.filename}</p>
          </div>
          <Chip tone={entry.required ? "neutral" : "secondary"}>{entry.required ? "Requires this" : "Optional"}</Chip>
        </li>
      ))}
    </ul>
  );
}
