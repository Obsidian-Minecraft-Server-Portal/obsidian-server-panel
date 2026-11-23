import type { DetectedSide, LeaningSide } from "../../api/serverpack";
import { SIDE_LABELS } from "./model";

const TONE: Record<DetectedSide, string> = {
  client: "bg-secondary-tint text-secondary-soft",
  server: "bg-accent-tint text-accent",
  both: "bg-raised text-ink-2",
  unsure: "bg-amber/12 text-amber",
};

const LEAN_PHRASES: Record<LeaningSide, string> = {
  client: "client only",
  server: "server only",
  both: "both sides",
};

const CHIP = "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 font-mono text-[11px] tracking-wider uppercase";

interface SideChipProps {
  side: DetectedSide;
  leaning: LeaningSide | null;
  confidence: number;
}

export function SideChip({ side, leaning, confidence }: SideChipProps) {
  const percent = Math.round(confidence * 100);

  if (side === "unsure") {
    return (
      <div className="flex flex-col items-start gap-1">
        <span className={`${CHIP} ${TONE.unsure}`}>{SIDE_LABELS.unsure}</span>
        <span className="font-mono text-[11px] text-ink-2">
          {leaning ? `${percent}% confident it's ${LEAN_PHRASES[leaning]}` : "No clear lean"}
        </span>
      </div>
    );
  }

  return (
    <span className={`${CHIP} ${TONE[side]}`} title={`${percent}% confident`}>
      {SIDE_LABELS[side]}
      <span className="font-normal tracking-normal text-ink-3 normal-case">{percent}%</span>
    </span>
  );
}
