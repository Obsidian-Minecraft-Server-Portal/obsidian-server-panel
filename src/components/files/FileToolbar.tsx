import { Icon } from "@iconify-icon/react";
import { useRef } from "react";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import type { FileAction } from "./fileActions";

interface FileToolbarProps {
  actions: FileAction[];
  refreshing: boolean;
  onNewFolder: () => void;
  onNewFile: () => void;
  onRefresh: () => void;
  onUpload: (files: File[]) => void;
}

export function FileToolbar({ actions, refreshing, onNewFolder, onNewFile, onRefresh, onUpload }: FileToolbarProps) {
  const input = useRef<HTMLInputElement>(null);

  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <PrimaryButton size="sm" onPress={() => input.current?.click()}>
          <Icon icon="lucide:upload" width={15} height={15} />
          Upload
        </PrimaryButton>
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
        <input
          ref={input}
          type="file"
          multiple
          hidden
          onChange={(event) => {
            const files = Array.from(event.target.files ?? []);
            event.target.value = "";
            if (files.length > 0) onUpload(files);
          }}
        />
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
