import { cn } from "@heroui/react";
import { toneDot, type Tone } from "./tone";

interface StatusDotProps {
  tone?: Tone;
  pulse?: boolean;
  className?: string;
}

export function StatusDot({ tone = "accent", pulse = false, className }: StatusDotProps) {
  return (
    <span className={cn("relative inline-flex size-2 shrink-0", className)} aria-hidden>
      {pulse && (
        <span className={cn("absolute inset-0 rounded-full animate-pulse-ring", toneDot[tone])} />
      )}
      <span className={cn("relative size-2 rounded-full", toneDot[tone])} />
    </span>
  );
}
