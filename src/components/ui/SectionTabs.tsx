import { cn } from "@heroui/react";

export interface SectionTab<T extends string> {
  id: T;
  label: string;
}

interface SectionTabsProps<T extends string> {
  label: string;
  tabs: SectionTab<T>[];
  value: T;
  onChange: (id: T) => void;
  className?: string;
}

export function SectionTabs<T extends string>({ label, tabs, value, onChange, className }: SectionTabsProps<T>) {
  return (
    <div
      role="tablist"
      aria-label={label}
      className={cn("inline-flex rounded-control border border-line bg-input p-1", className)}
    >
      {tabs.map((tab) => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          aria-selected={value === tab.id}
          onClick={() => onChange(tab.id)}
          className={cn(
            "rounded-[9px] px-4 py-1.5 text-sm font-medium transition-colors",
            value === tab.id
              ? "bg-raised text-ink-0 shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]"
              : "text-ink-3 hover:text-ink-1",
          )}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
