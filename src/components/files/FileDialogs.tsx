import type { FileEntry } from "../../api/files";
import { ConfirmDialog } from "./ConfirmDialog";
import { NameDialog } from "./NameDialog";
import { baseName, isSafeRelativeDir, isValidName, joinPath, normalizeDir, withZipExtension } from "./fileUtils";
import type { FileOps } from "./useFileOps";

export type FileDialog =
  | { kind: "newFolder" }
  | { kind: "newFile" }
  | { kind: "rename"; entry: FileEntry }
  | { kind: "move"; entries: FileEntry[] }
  | { kind: "archive"; entries: FileEntry[] }
  | { kind: "delete"; entries: FileEntry[] }
  | { kind: "overwrite"; names: string[]; confirm: () => void };

interface FileDialogsProps {
  dialog: FileDialog;
  dir: string;
  entries: FileEntry[];
  ops: FileOps;
  onOpenFile: (path: string) => void;
  onClose: () => void;
}

function nameProblem(value: string, taken: Set<string>, unchanged?: string): string | null {
  if (!isValidName(value)) return "Enter a valid name without slashes";
  if (value === unchanged) return "The name is unchanged";
  if (taken.has(value)) return "An entry with this name already exists";
  return null;
}

function moveProblem(destination: string, dir: string, entries: FileEntry[]): string | null {
  if (!isSafeRelativeDir(destination)) return "Paths cannot go above the instance root";
  if (destination === dir) return "The items are already in this folder";
  const intoItself = entries.some(
    (entry) => entry.is_dir && (destination === entry.path || destination.startsWith(`${entry.path}/`)),
  );
  return intoItself ? "A folder cannot be moved into itself" : null;
}

export function FileDialogs({ dialog, dir, entries, ops, onOpenFile, onClose }: FileDialogsProps) {
  const taken = new Set(entries.map((entry) => entry.name));

  switch (dialog.kind) {
    case "newFolder":
      return (
        <NameDialog
          title="New folder"
          icon="lucide:folder-plus"
          label="Folder name"
          initialValue=""
          submitLabel="Create"
          validate={(value) => nameProblem(value, taken)}
          onSubmit={(name) => ops.createFolder(joinPath(dir, name))}
          onClose={onClose}
        />
      );
    case "newFile":
      return (
        <NameDialog
          title="New file"
          icon="lucide:file-plus"
          label="File name"
          initialValue=""
          submitLabel="Create"
          validate={(value) => nameProblem(value, taken)}
          onSubmit={async (name) => {
            const path = joinPath(dir, name);
            await ops.createFile(path);
            onOpenFile(path);
          }}
          onClose={onClose}
        />
      );
    case "rename":
      return (
        <NameDialog
          title="Rename"
          icon="lucide:pencil"
          label="New name"
          initialValue={dialog.entry.name}
          submitLabel="Rename"
          validate={(value) => nameProblem(value, taken, dialog.entry.name)}
          onSubmit={(name) => ops.rename(dialog.entry.path, joinPath(dir, name))}
          onClose={onClose}
        />
      );
    case "move":
      return (
        <NameDialog
          title={`Move ${dialog.entries.length} ${dialog.entries.length === 1 ? "item" : "items"}`}
          icon="lucide:move"
          label="Destination folder (empty for the instance root)"
          initialValue={dir}
          submitLabel="Move"
          validate={(value) => moveProblem(normalizeDir(value), dir, dialog.entries)}
          onSubmit={(destination) => ops.move(dialog.entries.map((entry) => entry.path), normalizeDir(destination))}
          onClose={onClose}
        />
      );
    case "archive": {
      const [single] = dialog.entries;
      const initial = dialog.entries.length === 1 ? baseName(single.path).replace(/\.[^.]*$/, "") : "archive";
      return (
        <NameDialog
          title="Compress to zip"
          icon="lucide:file-archive"
          label="Archive name"
          initialValue={initial}
          submitLabel="Compress"
          validate={(value) => (value ? nameProblem(withZipExtension(value), taken) : "Enter a name")}
          onSubmit={(name) =>
            ops.archive(
              dialog.entries.map((entry) => entry.path),
              joinPath(dir, withZipExtension(name)),
            )
          }
          onClose={onClose}
        />
      );
    }
    case "overwrite": {
      const single = dialog.names.length === 1;
      return (
        <ConfirmDialog
          title={`Replace ${single ? dialog.names[0] : `${dialog.names.length} files`}?`}
          description={
            single
              ? "A file with this name already exists in this folder. Uploading replaces it."
              : `These files already exist in this folder and uploading replaces them: ${dialog.names.join(", ")}.`
          }
          confirmLabel="Replace"
          danger
          onConfirm={async () => dialog.confirm()}
          onClose={onClose}
        />
      );
    }
    case "delete": {
      const count = dialog.entries.length;
      return (
        <ConfirmDialog
          title={`Delete ${count === 1 ? dialog.entries[0].name : `${count} items`}?`}
          description={
            <>
              <p>This permanently removes {count === 1 ? "this entry" : "these entries"} and everything inside folders. It cannot be undone.</p>
              {count > 1 && (
                <ul className="max-h-40 list-disc overflow-y-auto pl-5 font-mono text-xs text-ink-3">
                  {dialog.entries.map((entry) => (
                    <li key={entry.path}>{entry.name}</li>
                  ))}
                </ul>
              )}
            </>
          }
          confirmLabel="Delete"
          danger
          onConfirm={() => ops.remove(dialog.entries.map((entry) => entry.path))}
          onClose={onClose}
        />
      );
    }
  }
}
