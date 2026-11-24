import { Icon } from "@iconify-icon/react";
import { useRef, type ChangeEvent, type InputHTMLAttributes } from "react";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import type { FileAction } from "./fileActions";
import { readUploadEntries, type UploadEntry } from "./uploadEntries";

interface FileToolbarProps {
  actions: FileAction[];
  refreshing: boolean;
  onNewFolder: () => void;
  onNewFile: () => void;
  onRefresh: () => void;
  onUpload: (entries: UploadEntry[]) => void;
}

const directoryPicker = { webkitdirectory: "" } as InputHTMLAttributes<HTMLInputElement>;

export function FileToolbar({ actions, refreshing, onNewFolder, onNewFile, onRefresh, onUpload }: FileToolbarProps) {
  const files = useRef<HTMLInputElement>(null);
  const folders = useRef<HTMLInputElement>(null);

  const pick = async (event: ChangeEvent<HTMLInputElement>) => {
    const entries = await readUploadEntries(event.nativeEvent);
    event.target.value = "";
    if (entries.length > 0) onUpload(entries);
  };

  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <PrimaryButton size="sm" onPress={() => files.current?.click()}>
          <Icon icon="lucide:upload" width={15} height={15} />
          Upload
        </PrimaryButton>
        <GhostButton size="sm" onPress={() => folders.current?.click()}>
          <Icon icon="lucide:folder-up" width={15} height={15} />
          Upload folder
        </GhostButton>
        <GhostButton size="sm" onPress={onNewFolder}>
          <Icon icon="lucide:folder-plus" width={15} height={15} />
          New folder
        </GhostButton>
        <GhostButton size="sm" onPress={onNewFile}>
          <Icon icon="lucide:file-plus" width={15} height={15} />
          New file
        </GhostButton>
        <GhostButton size="sm" aria-label="Refresh" isDisabled={refreshing} onPress={onRefresh}>
          <Icon icon="lucide:refresh-cw" className={refreshing ? "animate-spin" : undefined} width={15} height={15} />
        </GhostButton>
        <input ref={files} type="file" multiple hidden onChange={pick} />
        <input ref={folders} type="file" hidden {...directoryPicker} onChange={pick} />
      </div>
      {actions.length > 0 && (
        <div className="flex flex-wrap items-center gap-2">
          {actions.map((action) => (
            <GhostButton
              key={action.key}
              size="sm"
              onPress={action.run}
              className={action.danger ? "text-rose hover:border-rose" : undefined}
            >
              <Icon icon={action.icon} width={15} height={15} />
              {action.label}
            </GhostButton>
          ))}
        </div>
      )}
    </div>
  );
}
