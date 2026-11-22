import type { DetectedSide } from "../../api/serverpack";
import { SIDE_LABELS } from "./model";

const TONE: Record<DetectedSide, string> = {
  client: "bg-cyan/12 text-cyan",
  server: "bg-accent-tint text-accent",
  both: "bg-raised text-ink-2",
  unsure: "bg-amber/12 text-amber",
};

export function SideChip({ side, confidence }: { side: DetectedSide; confidence: number }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 font-mono text-[11px] tracking-wider uppercase ${TONE[side]}`}
      title={`${Math.round(confidence * 100)}% confident`}
    >
      {SIDE_LABELS[side]}
      <span className="font-normal tracking-normal text-ink-3 normal-case">{Math.round(confidence * 100)}%</span>
    </span>
  );
}
