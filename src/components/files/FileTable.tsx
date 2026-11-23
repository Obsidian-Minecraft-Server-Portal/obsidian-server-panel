import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { memo, type MouseEvent } from "react";
import type { FileEntry } from "../../api/files";
import { formatBytes, formatModified, isArchive } from "./fileUtils";

interface Modifiers {
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
}

interface FileTableProps {
  entries: FileEntry[];
  selected: ReadonlySet<string>;
  onPick: (path: string, modifiers: Modifiers) => void;
  onOpen: (entry: FileEntry) => void;
  onContextMenu: (entry: FileEntry, x: number, y: number) => void;
  compact?: boolean;
}

export function FileTable({ entries, selected, onPick, onOpen, onContextMenu, compact = false }: FileTableProps) {
  return (
    <div className={compact ? undefined : "overflow-x-auto"}>
      <table className={cn("w-full border-collapse text-left text-sm", compact ? "table-fixed" : "sm:min-w-[32rem]")} aria-multiselectable="true">
        <thead>
          <tr className="border-b border-line font-mono text-[11px] tracking-wider text-ink-3 uppercase">
            <th scope="col" className="px-4 py-2.5 font-medium">Name</th>
            {!compact && (
              <>
                <th scope="col" className="w-28 px-4 py-2.5 text-right font-medium">Size</th>
                <th scope="col" className="hidden w-48 px-4 py-2.5 font-medium sm:table-cell">Modified</th>
              </>
            )}
          </tr>
        </thead>
        <tbody>
          {entries.map((entry) => (
            <FileRow
              key={entry.path}
              entry={entry}
              selected={selected.has(entry.path)}
              onPick={onPick}
              onOpen={onOpen}
              onContextMenu={onContextMenu}
              compact={compact}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}

interface FileRowProps extends Omit<FileTableProps, "entries" | "selected" | "compact"> {
  entry: FileEntry;
  selected: boolean;
  compact: boolean;
}

const FileRow = memo(function FileRow({ entry, selected, onPick, onOpen, onContextMenu, compact }: FileRowProps) {
  const handleClick = (event: MouseEvent) => onPick(entry.path, event);
  const handleContext = (event: MouseEvent) => {
    event.preventDefault();
    onContextMenu(entry, event.clientX, event.clientY);
  };

  return (
    <tr
      tabIndex={0}
      aria-selected={selected}
      onClick={handleClick}
      onDoubleClick={() => onOpen(entry)}
      onKeyDown={(event) => event.key === "Enter" && onOpen(entry)}
      onContextMenu={handleContext}
      className={cn(
        "cursor-default border-b border-line/60 transition-colors outline-none select-none focus-visible:bg-raised",
        selected ? "bg-accent-tint/60" : "hover:bg-raised/60",
      )}
    >
      <td className="px-4 py-2.5">
        <span className="flex min-w-0 items-center gap-3">
          <Icon icon={iconFor(entry)} className={cn("shrink-0", entry.is_dir ? "text-secondary" : "text-ink-3")} width={16} height={16} />
          <span className="truncate text-ink-1">{entry.name}</span>
          {entry.is_symlink && <span className="font-mono text-[10px] text-ink-4 uppercase">link</span>}
        </span>
      </td>
      {!compact && (
        <>
          <td className="px-4 py-2.5 text-right font-mono text-xs text-ink-3 tabular-nums">
            {entry.is_dir ? "—" : formatBytes(entry.size)}
          </td>
          <td className="hidden px-4 py-2.5 font-mono text-xs text-ink-3 sm:table-cell">{formatModified(entry.modified)}</td>
        </>
      )}
    </tr>
  );
});

function iconFor(entry: FileEntry): string {
  if (entry.is_dir) return "lucide:folder";
  if (entry.is_symlink) return "lucide:link-2";
  if (isArchive(entry.name)) return "lucide:file-archive";
  return "lucide:file";
}
