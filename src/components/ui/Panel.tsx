import { cn } from "@heroui/react";
import type { ReactNode } from "react";

interface PanelProps {
  title?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function Panel({ title, actions, children, className }: PanelProps) {
  return (
    <section className={cn("rounded-panel border border-line bg-panel p-5", className)}>
      {(title || actions) && (
        <header className="mb-4 flex items-center justify-between gap-3">
          {title && <h2 className="text-base font-semibold text-ink-0">{title}</h2>}
          {actions}
        </header>
      )}
      {children}
    </section>
  );
}
