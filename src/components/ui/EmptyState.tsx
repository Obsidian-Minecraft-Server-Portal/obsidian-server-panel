import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { ReactNode } from "react";

interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: ReactNode;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({ icon = "lucide:inbox", title, description, action, className }: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-4 rounded-panel border border-dashed border-line-strong px-6 py-16 text-center animate-tick-in",
        className,
      )}
    >
      <span className="grid size-14 place-items-center rounded-2xl bg-secondary-tint text-secondary">
        <Icon icon={icon} width={28} height={28} />
      </span>
      <div className="space-y-1.5">
        <h3 className="text-lg font-semibold text-ink-0">{title}</h3>
        {description && <p className="max-w-md text-sm text-ink-3">{description}</p>}
      </div>
      {action}
    </div>
  );
}
