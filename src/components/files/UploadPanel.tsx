import { Icon } from "@iconify-icon/react";
import { ProgressBar } from "../ui/ProgressBar";
import { GhostButton } from "../ui/GhostButton";
import { formatBytes } from "./fileUtils";
import { uploadQueue, useUploadItems, type UploadState } from "./useUploads";

const STATE_LABEL: Record<UploadState, string> = {
  queued: "Queued",
  uploading: "Uploading",
  done: "Done",
  failed: "Failed",
  cancelled: "Cancelled",
};

const STATE_TEXT: Record<UploadState, string> = {
  queued: "text-ink-3",
  uploading: "text-secondary",
  done: "text-green",
  failed: "text-rose",
  cancelled: "text-ink-3",
};

export function UploadPanel() {
  const uploads = useUploadItems();
  if (uploads.length === 0) return null;
  const active = uploads.filter((item) => item.state === "queued" || item.state === "uploading").length;

  return (
    <section
      aria-label="Uploads"
      className="fixed right-5 bottom-5 z-40 flex max-h-[60vh] w-[22rem] max-w-[calc(100vw-2.5rem)] flex-col overflow-hidden rounded-card border border-line-strong bg-panel shadow-[0_30px_80px_-20px_rgba(0,0,0,0.7)]"
    >
      <header className="flex items-center justify-between gap-3 border-b border-line px-4 py-3">
        <p className="font-mono text-[11px] tracking-wider text-ink-2 uppercase">
          Uploads{active > 0 ? ` · ${active} active` : ""}
        </p>
        <GhostButton size="sm" isDisabled={active === uploads.length} onPress={uploadQueue.clearFinished}>
          Clear finished
        </GhostButton>
      </header>
      <ul className="space-y-3 overflow-y-auto p-3">
        {uploads.map((item) => (
          <li key={item.key} className="space-y-1.5 rounded-control bg-input/60 p-3">
            <div className="flex items-center justify-between gap-2">
              <p className="min-w-0 truncate text-sm text-ink-1" title={item.path}>
                {item.name}
              </p>
              {(item.state === "queued" || item.state === "uploading") && (
                <GhostButton
                  isIconOnly
                  size="sm"
                  aria-label={`Cancel upload of ${item.name}`}
                  onPress={() => uploadQueue.cancel(item.key)}
                  className="size-7 shrink-0 text-ink-3 hover:text-rose"
                >
                  <Icon icon="lucide:x" width={14} height={14} />
                </GhostButton>
              )}
            </div>
            <ProgressBar value={item.progress * 100} />
            <div className="flex items-center justify-between font-mono text-[11px]">
              <span className={STATE_TEXT[item.state]}>
                {STATE_LABEL[item.state]}
                {item.state === "uploading" && ` · ${Math.round(item.progress * 100)}%`}
              </span>
              <span className="text-ink-3">{formatBytes(item.size)}</span>
            </div>
            {item.error && <p className="text-xs text-rose">{item.error}</p>}
          </li>
        ))}
      </ul>
    </section>
  );
}
