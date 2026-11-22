import { cn } from "@heroui/react";
import type { MemoryLimits } from "../../api/system";

interface MemoryRangeProps {
  limits: MemoryLimits;
  min: number;
  max: number;
  onChange: (min: number, max: number) => void;
  isDisabled?: boolean;
  step?: number;
}

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
  const track = `linear-gradient(90deg,
    var(--success) 0%, var(--success) ${greenEnd}%,
    var(--warning) ${greenEnd}%, var(--warning) ${yellowEnd}%,
    var(--danger) ${yellowEnd}%, var(--danger) 100%)`;

  const minPct = pct(min, total);
  const maxPct = pct(max, total);

  return (
    <div className="space-y-3">
      <div className="relative h-8">
        <div className="absolute inset-x-0 top-1/2 h-2 -translate-y-1/2 rounded-full opacity-90" style={{ background: track }} />
        <div
          className="absolute top-1/2 h-4 -translate-y-1/2 rounded-full border-2 border-accent bg-accent/25"
          style={{ left: `${minPct}%`, width: `${Math.max(0, maxPct - minPct)}%` }}
          aria-hidden
        />
        <input
          type="range"
          aria-label="Minimum memory"
          min={0}
          max={total}
          step={step}
          value={min}
          disabled={isDisabled}
          onChange={(event) => onChange(Math.min(Number(event.target.value), max), max)}
          className="memory-range"
        />
        <input
          type="range"
          aria-label="Maximum memory"
          min={0}
          max={total}
          step={step}
          value={max}
          disabled={isDisabled}
          onChange={(event) => onChange(min, Math.max(Number(event.target.value), min))}
          className="memory-range"
        />
      </div>
      <div className="flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] text-ink-3">
        <span>
          Min <span className="text-ink-1">{formatGb(min)}</span> · Max <span className="text-ink-1">{formatGb(max)}</span>
        </span>
        <span className="flex items-center gap-3">
          <Legend className="bg-success" label={`Recommended ≤ ${formatGb(limits.recommended_mb)}`} />
          <Legend className="bg-warning" label={`Above recommended`} />
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
