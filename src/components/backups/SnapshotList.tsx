import { cn } from "@heroui/react";
import type { SnapshotInfo } from "../../api/backups";
import { snapshotDownloadUrl } from "../../api/backups";
import { Chip } from "../ui/Chip";
import { ConfirmButton } from "./ConfirmButton";
import { GhostButton } from "../ui/GhostButton";
import { formatBytes, formatDateTime, labelTone } from "./format";

interface SnapshotListProps {
  instanceId: number;
  snapshots: SnapshotInfo[];
  browsing: string | null;
  disabled: boolean;
  onBrowse: (snapshot: string) => void;
  onRestore: (snapshot: string) => void;
  onDelete: (snapshot: string) => void;
}

export function SnapshotList({ instanceId, snapshots, browsing, disabled, onBrowse, onRestore, onDelete }: SnapshotListProps) {
  return (
    <ul className="divide-y divide-line">
      {snapshots.map((snapshot) => (
        <li
          key={snapshot.id}
          className={cn("flex flex-wrap items-center gap-3 py-3", browsing === snapshot.id && "bg-raised/60")}
        >
          <div className="min-w-0 flex-1 space-y-1">
            <div className="flex items-center gap-2">
              <span className="truncate font-mono text-xs text-ink-1">{snapshot.id.slice(0, 16)}</span>
              <Chip tone={labelTone(snapshot.label)}>{snapshot.label}</Chip>
            </div>
            <p className="font-mono text-[11px] text-ink-3">
              {formatDateTime(snapshot.created_at)} · {snapshot.file_count} files · {formatBytes(snapshot.size_bytes)}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-1.5">
            <GhostButton size="sm" onPress={() => onBrowse(snapshot.id)}>
              Browse
            </GhostButton>
            <a
              href={snapshotDownloadUrl(instanceId, snapshot.id)}
              download
              className="inline-flex h-8 items-center rounded-control border border-line-strong bg-input px-3 text-sm font-medium text-ink-1 hover:bg-raised"
            >
              Zip
            </a>
            <ConfirmButton
              label="Restore"
              confirmLabel="Confirm restore"
              isDisabled={disabled}
              onConfirm={() => onRestore(snapshot.id)}
            />
            <ConfirmButton
              label="Delete"
              confirmLabel="Confirm delete"
              danger
              isDisabled={disabled}
              onConfirm={() => onDelete(snapshot.id)}
            />
          </div>
        </li>
      ))}
    </ul>
  );
}
