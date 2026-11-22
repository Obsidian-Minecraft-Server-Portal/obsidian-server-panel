import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import type { InstanceStatus } from "../../api/instances";
import { GhostButton } from "../ui/GhostButton";

export function CrashHints({ status }: { status: InstanceStatus }) {
  const hints = status.crash_hints ?? [];
  const key = hints.join("\n");
  const [dismissed, setDismissed] = useState<string | null>(null);
  if (status.state !== "crashed" || hints.length === 0 || dismissed === key) return null;

  return (
    <section role="alert" className="mx-8 mt-6 flex gap-3 rounded-panel border border-rose/40 bg-rose/10 px-4 py-3">
      <Icon icon="lucide:triangle-alert" width={18} height={18} className="mt-0.5 shrink-0 text-rose" />
      <div className="min-w-0 flex-1 space-y-2">
        <p className="font-mono text-[11px] tracking-wider text-rose uppercase">Crash hints</p>
        <ul className="list-disc space-y-1 pl-4 text-sm text-ink-1">
          {hints.map((hint, index) => (
            <li key={index}>{hint}</li>
          ))}
        </ul>
      </div>
      <GhostButton isIconOnly size="sm" aria-label="Dismiss crash hints" onPress={() => setDismissed(key)}>
        <Icon icon="lucide:x" width={14} height={14} />
      </GhostButton>
    </section>
  );
}
