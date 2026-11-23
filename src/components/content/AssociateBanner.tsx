import { Icon } from "@iconify-icon/react";
import { GhostButton } from "../ui/GhostButton";

interface AssociateBannerProps {
  filename: string;
  onCancel: () => void;
}

export function AssociateBanner({ filename, onCancel }: AssociateBannerProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 rounded-panel border border-secondary/40 bg-secondary-tint px-4 py-3">
      <p className="flex min-w-0 items-center gap-2 text-sm text-ink-1">
        <Icon icon="lucide:link" width={16} height={16} className="shrink-0 text-secondary" />
        <span className="min-w-0">
          Associating <span className="font-mono text-xs">{filename}</span>. Pick the project it came from, then choose its version.
        </span>
      </p>
      <GhostButton size="sm" onPress={onCancel}>
        Cancel
      </GhostButton>
    </div>
  );
}
