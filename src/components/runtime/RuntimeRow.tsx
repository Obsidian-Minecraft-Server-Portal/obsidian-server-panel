import { format } from "date-fns";
import type { JavaRuntime } from "../../api/java";
import { Chip } from "../ui/Chip";
import { GhostButton } from "../ui/GhostButton";

interface RuntimeRowProps {
  runtime: JavaRuntime;
  canManage: boolean;
  confirming: boolean;
  removing: boolean;
  onRemove: () => void;
  onCancel: () => void;
}

export function RuntimeRow({ runtime, canManage, confirming, removing, onRemove, onCancel }: RuntimeRowProps) {
  return (
    <div className="flex flex-wrap items-center gap-x-6 gap-y-3 border-t border-line py-4 first:border-t-0 first:pt-0">
      <div className="flex min-w-48 flex-1 flex-col gap-1.5">
        <div className="flex flex-wrap items-center gap-2">
          <span className="font-mono text-sm font-semibold text-ink-0">Java {runtime.version}</span>
          <Chip tone={runtime.source === "piston" ? "secondary" : "cyan"}>
            {runtime.source === "piston" ? "Managed" : "System"}
          </Chip>
          <Chip>{runtime.arch}</Chip>
        </div>
        <p className="truncate font-mono text-xs text-ink-3" title={runtime.path}>
          {runtime.component ?? "detected"} · {runtime.path}
        </p>
      </div>
      <span className="font-mono text-xs text-ink-4">{format(new Date(runtime.installed_at), "d MMM yyyy")}</span>
      {canManage && (
        <div className="flex items-center gap-2">
          {confirming && (
            <GhostButton size="sm" onPress={onCancel} isDisabled={removing}>
              Cancel
            </GhostButton>
          )}
          <GhostButton
            size="sm"
            isPending={removing}
            onPress={onRemove}
            className={confirming ? "border-rose/40 text-rose" : undefined}
          >
            {confirming ? "Confirm remove" : "Remove"}
          </GhostButton>
        </div>
      )}
    </div>
  );
}
