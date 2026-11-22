import { cn } from "@heroui/react";
import type { ReactNode } from "react";
import { StatusDot } from "./StatusDot";
import { toneText, type Tone } from "./tone";

interface ChipProps {
  children: ReactNode;
  tone?: Tone;
  pulse?: boolean;
  className?: string;
}

export function Chip({ children, tone = "neutral", pulse, className }: ChipProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 font-mono text-[11px] font-medium uppercase tracking-wider",
        toneText[tone],
        className,
      )}
    >
      {pulse && <StatusDot tone={tone} pulse />}
      {children}
    </span>
  );
}
