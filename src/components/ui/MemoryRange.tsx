import { Slider, cn } from "@heroui/react";
import type { MemoryLimits } from "../../api/system";

interface MemoryRangeProps {
  limits: MemoryLimits;
  min: number;
  max: number;
  onChange: (min: number, max: number) => void;
  isDisabled?: boolean;
  step?: number;
}

const THUMB_CLASS =
  "size-[18px] cursor-grab rounded-full border-2 border-ink-0 bg-accent shadow-[0_0_0_3px_var(--accent-tint),0_0_14px_var(--accent-glow)] data-[dragging=true]:cursor-grabbing data-[disabled=true]:opacity-50";

function pct(mb: number, total: number): number {
  return Math.min(100, Math.max(0, (mb / total) * 100));
}

function formatGb(mb: number): string {
  return mb >= 1024 ? `${(mb / 1024).toFixed(mb % 1024 === 0 ? 0 : 1)} GB` : `${mb} MB`;
}

export function MemoryRange({ limits, min, max, onChange, isDisabled, step = 256 }: MemoryRangeProps) {
  const total = limits.total_mb;
  const greenEnd = pct(limits.recommended_mb, total);
  const yellowEnd = pct(limits.usable_mb, total);
  const zones = `linear-gradient(90deg,
    var(--success) 0%, var(--success) ${greenEnd}%,
    var(--warning) ${greenEnd}%, var(--warning) ${yellowEnd}%,
    var(--danger) ${yellowEnd}%, var(--danger) 100%)`;

  return (
    <div className="space-y-3">
      <Slider
        aria-label="Memory"
        minValue={0}
        maxValue={total}
        step={step}
        value={[min, max]}
        isDisabled={isDisabled}
        onChange={(value) => {
          const [low, high] = Array.isArray(value) ? value : [min, max];
          onChange(low, high);
        }}
        className="w-full"
      >
        <Slider.Track className="relative h-8 bg-transparent">
          <span
            aria-hidden
            className="pointer-events-none absolute inset-x-0 top-1/2 h-2 -translate-y-1/2 rounded-full opacity-90"
            style={{ background: zones }}
          />
          <Slider.Fill className={cn("top-1/2 h-4 -translate-y-1/2 rounded-full border-2 border-accent bg-accent/25")} />
          <Slider.Thumb index={0} aria-label="Minimum memory" className={THUMB_CLASS} />
          <Slider.Thumb index={1} aria-label="Maximum memory" className={THUMB_CLASS} />
        </Slider.Track>
      </Slider>
      <div className="flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] text-ink-3">
        <span>
          Min <span className="text-ink-1">{formatGb(min)}</span> · Max <span className="text-ink-1">{formatGb(max)}</span>
        </span>
        <span className="flex flex-wrap items-center gap-3">
          <Legend className="bg-success" label={`Recommended ≤ ${formatGb(limits.recommended_mb)}`} />
          <Legend className="bg-warning" label="Above recommended" />
          <Legend className="bg-danger" label={`Above usable ${formatGb(limits.usable_mb)}`} />
        </span>
      </div>
      <p className="text-xs text-ink-4">
        Host has {formatGb(total)}; {formatGb(total - limits.usable_mb)} is kept for the OS and the panel.
      </p>
    </div>
  );
}

function Legend({ className, label }: { className: string; label: string }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className={cn("size-2 rounded-full", className)} />
      {label}
    </span>
  );
}
