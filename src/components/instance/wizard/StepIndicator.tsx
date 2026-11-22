import { Button, cn } from "@heroui/react";
import { STEP_LABELS } from "./draft";

interface StepIndicatorProps {
  current: number;
  onSelect: (step: number) => void;
  maxReachable: number;
}

export function StepIndicator({ current, onSelect, maxReachable }: StepIndicatorProps) {
  return (
    <ol className="flex flex-wrap gap-2" aria-label="Setup steps">
      {STEP_LABELS.map((label, index) => {
        const active = index === current;
        const reachable = index <= maxReachable;
        return (
          <li key={label}>
            <Button
              type="button"
              isDisabled={!reachable}
              aria-current={active ? "step" : undefined}
              onPress={() => onSelect(index)}
              className={cn(
                "flex items-center gap-2 rounded-full border px-3.5 py-1.5 text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-40",
                active ? "border-accent bg-accent-tint text-ink-0" : "border-line bg-panel text-ink-3 hover:text-ink-1",
              )}
            >
              <span className={cn("grid size-5 place-items-center rounded-full font-mono text-[11px]", active ? "bg-accent text-accent-ink" : "bg-raised")}>
                {index + 1}
              </span>
              {label}
            </Button>
          </li>
        );
      })}
    </ol>
  );
}
