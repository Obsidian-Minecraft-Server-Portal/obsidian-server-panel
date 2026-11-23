import { Tooltip } from "@heroui/react";
import type { ReactNode } from "react";

interface TipProps {
  label: string;
  placement?: "top" | "right" | "bottom" | "left";
  children: ReactNode;
}

export function Tip({ label, placement = "right", children }: TipProps) {
  return (
    <Tooltip delay={0} closeDelay={0}>
      <Tooltip.Trigger>{children}</Tooltip.Trigger>
      <Tooltip.Content
        placement={placement}
        offset={10}
        className="rounded-control border border-line-strong bg-raised px-2.5 py-1.5 text-xs font-medium text-ink-0 shadow-lift"
      >
        {label}
      </Tooltip.Content>
    </Tooltip>
  );
}
