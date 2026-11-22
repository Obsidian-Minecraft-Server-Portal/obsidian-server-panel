import { Icon } from "@iconify-icon/react";
import { useState, type DragEvent, type ReactNode } from "react";

interface FileDropZoneProps {
  label: string;
  onFiles: (files: File[]) => void;
  children: ReactNode;
}

function hasFiles(event: DragEvent): boolean {
  return Array.from(event.dataTransfer.types).includes("Files");
}

export function FileDropZone({ label, onFiles, children }: FileDropZoneProps) {
  const [depth, setDepth] = useState(0);

  const enter = (event: DragEvent) => {
    if (!hasFiles(event)) return;
    event.preventDefault();
    setDepth((value) => value + 1);
  };

  const leave = (event: DragEvent) => {
    if (!hasFiles(event)) return;
    setDepth((value) => Math.max(0, value - 1));
  };

  const drop = (event: DragEvent) => {
    if (!hasFiles(event)) return;
    event.preventDefault();
    setDepth(0);
    const files = Array.from(event.dataTransfer.files);
    if (files.length > 0) onFiles(files);
  };

  return (
    <div
      className="relative"
      onDragEnter={enter}
      onDragOver={(event) => hasFiles(event) && event.preventDefault()}
      onDragLeave={leave}
      onDrop={drop}
    >
      {children}
      {depth > 0 && (
        <div className="pointer-events-none absolute inset-0 z-10 grid place-items-center rounded-panel border-2 border-dashed border-accent bg-canvas/80 backdrop-blur-sm">
          <div className="flex flex-col items-center gap-2 text-center">
            <Icon icon="lucide:upload-cloud" className="text-accent" width={32} height={32} />
            <p className="text-sm font-medium text-ink-0">Drop files to upload into {label}</p>
          </div>
        </div>
      )}
    </div>
  );
}
