import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { ReactNode } from "react";

interface SearchProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

interface PageHeaderProps {
  title: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  search?: SearchProps;
  className?: string;
}

export function PageHeader({ title, subtitle, actions, search, className }: PageHeaderProps) {
  return (
    <header className={cn("mb-6 flex flex-wrap items-center justify-between gap-4 border-b border-line px-8 py-6", className)}>
      <div className="min-w-0 space-y-1">
        <h1 className="truncate text-3xl font-semibold tracking-tight text-ink-0">{title}</h1>
        {subtitle && <p className="text-sm text-ink-3">{subtitle}</p>}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {search && (
          <label className="relative flex w-60 items-center">
            <Icon icon="lucide:search" className="pointer-events-none absolute left-3 text-ink-3" width={16} height={16} />
            <input
              type="search"
              value={search.value}
              placeholder={search.placeholder ?? "Search"}
              onChange={(event) => search.onChange(event.target.value)}
              className="h-9 w-full rounded-control border border-line bg-input pr-3 pl-9 text-sm text-ink-1 placeholder:text-ink-4 focus:border-accent"
            />
          </label>
        )}
        {actions}
      </div>
    </header>
  );
}
