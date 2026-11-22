import type { ReactNode } from "react";

interface FormSectionProps {
  index: number;
  title: ReactNode;
  description?: string;
  children: ReactNode;
}

export function FormSection({ index, title, description, children }: FormSectionProps) {
  return (
    <section className="rounded-panel border border-line bg-gradient-to-b from-panel to-rail p-6 glass-hairline">
      <header className="mb-5 flex items-start gap-4">
        <span className="grid size-8 shrink-0 place-items-center rounded-[10px] border border-accent/40 bg-accent-tint font-mono text-xs font-semibold text-accent">
          {String(index).padStart(2, "0")}
        </span>
        <div className="space-y-0.5">
          <h2 className="text-base font-semibold text-ink-0">{title}</h2>
          {description && <p className="text-sm text-ink-3">{description}</p>}
        </div>
      </header>
      {children}
    </section>
  );
}
