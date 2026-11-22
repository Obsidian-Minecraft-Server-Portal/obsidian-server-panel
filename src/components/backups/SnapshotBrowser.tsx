import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { Fragment, useState } from "react";
import { useSnapshotTree, type SnapshotInfo, type TreeEntry } from "../../api/backups";
import { EmptyState } from "../ui/EmptyState";
import { GhostButton } from "../ui/GhostButton";
import { Panel } from "../ui/Panel";
import { ConfirmButton } from "./ConfirmButton";
import { formatBytes } from "./format";

interface SnapshotBrowserProps {
  instanceId: number;
  snapshot: SnapshotInfo;
  disabled: boolean;
  onClose: () => void;
  onRestore: (path: string) => void;
}

export function SnapshotBrowser({ instanceId, snapshot, disabled, onClose, onRestore }: SnapshotBrowserProps) {
  const [path, setPath] = useState("");
  const tree = useSnapshotTree(instanceId, snapshot.id, path);
  const crumbs = path ? path.split("/") : [];

  return (
    <Panel
      title={`Browse ${snapshot.id.slice(0, 16)}`}
      actions={
        <div className="flex items-center gap-1.5">
          {path && (
            <ConfirmButton
              label="Restore folder"
              confirmLabel="Confirm restore"
              isDisabled={disabled}
              onConfirm={() => onRestore(path)}
            />
          )}
          <GhostButton size="sm" onPress={onClose}>
            Close
          </GhostButton>
        </div>
      }
    >
      <nav className="mb-4 flex flex-wrap items-center gap-1 font-mono text-xs text-ink-3">
        <Button size="sm" variant="ghost" onPress={() => setPath("")} className="h-7 min-w-0 px-2 text-ink-2">
          root
        </Button>
        {crumbs.map((crumb, index) => {
          const target = crumbs.slice(0, index + 1).join("/");
          return (
            <Fragment key={target}>
              <span aria-hidden>/</span>
              <Button size="sm" variant="ghost" onPress={() => setPath(target)} className="h-7 min-w-0 px-2 text-ink-2">
                {crumb}
              </Button>
            </Fragment>
          );
        })}
      </nav>

      {tree.isError && <p className="text-sm text-rose">{tree.error.message}</p>}
      {tree.data && tree.data.length === 0 && (
        <EmptyState icon="lucide:folder-open" title="Empty folder" />
      )}
      {tree.data && tree.data.length > 0 && (
        <ul className="divide-y divide-line">
          {tree.data.map((entry) => (
            <EntryRow
              key={entry.path}
              entry={entry}
              disabled={disabled}
              onOpen={() => setPath(entry.path)}
              onRestore={() => onRestore(entry.path)}
            />
          ))}
        </ul>
      )}
    </Panel>
  );
}

interface EntryRowProps {
  entry: TreeEntry;
  disabled: boolean;
  onOpen: () => void;
  onRestore: () => void;
}

function EntryRow({ entry, disabled, onOpen, onRestore }: EntryRowProps) {
  return (
    <li className="flex items-center gap-3 py-2.5">
      <Icon
        icon={entry.is_dir ? "lucide:folder" : "lucide:file"}
        width={16}
        height={16}
        className={entry.is_dir ? "shrink-0 text-accent" : "shrink-0 text-ink-3"}
      />
      {entry.is_dir ? (
        <Button variant="ghost" size="sm" onPress={onOpen} className="h-7 min-w-0 flex-1 justify-start truncate px-1 text-sm text-ink-1">
          {entry.name}
        </Button>
      ) : (
        <span className="min-w-0 flex-1 truncate text-sm text-ink-2">{entry.name}</span>
      )}
      <span className="font-mono text-[11px] text-ink-3">{formatBytes(entry.size_bytes)}</span>
      <ConfirmButton label="Restore" confirmLabel="Confirm" isDisabled={disabled} onConfirm={onRestore} />
    </li>
  );
}
