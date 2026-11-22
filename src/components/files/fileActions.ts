import type { FileEntry } from "../../api/files";
import { isArchive } from "./fileUtils";

export interface FileAction {
  key: string;
  label: string;
  icon: string;
  danger?: boolean;
  run: () => void;
}

export interface FileActionHandlers {
  open: (entry: FileEntry) => void;
  edit: (entry: FileEntry) => void;
  download: (entry: FileEntry) => void;
  rename: (entry: FileEntry) => void;
  move: (entries: FileEntry[]) => void;
  archive: (entries: FileEntry[]) => void;
  extract: (entry: FileEntry) => void;
  remove: (entries: FileEntry[]) => void;
}

export function actionsFor(entries: FileEntry[], handlers: FileActionHandlers): FileAction[] {
  if (entries.length === 0) return [];
  const [single] = entries;
  const actions: FileAction[] = [];

  if (entries.length === 1) {
    if (single.is_dir) {
      actions.push({ key: "open", label: "Open", icon: "lucide:folder-open", run: () => handlers.open(single) });
    } else {
      actions.push({ key: "edit", label: "Edit", icon: "lucide:file-pen-line", run: () => handlers.edit(single) });
    }
    actions.push({
      key: "download",
      label: single.is_dir ? "Download as zip" : "Download",
      icon: "lucide:download",
      run: () => handlers.download(single),
    });
    actions.push({ key: "rename", label: "Rename", icon: "lucide:pencil", run: () => handlers.rename(single) });
  }

  actions.push({ key: "move", label: "Move", icon: "lucide:move", run: () => handlers.move(entries) });
  actions.push({ key: "archive", label: "Compress", icon: "lucide:file-archive", run: () => handlers.archive(entries) });

  if (entries.length === 1 && !single.is_dir && isArchive(single.name)) {
    actions.push({ key: "extract", label: "Extract", icon: "lucide:package-open", run: () => handlers.extract(single) });
  }

  actions.push({ key: "delete", label: "Delete", icon: "lucide:trash-2", danger: true, run: () => handlers.remove(entries) });
  return actions;
}
