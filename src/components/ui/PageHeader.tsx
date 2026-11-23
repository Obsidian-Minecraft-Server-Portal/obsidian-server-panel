import { SearchField, cn } from "@heroui/react";
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
    <header className={cn("mb-6 flex flex-wrap items-center justify-between gap-4 border-b border-line px-4 py-6 sm:px-8", className)}>
      <div className="min-w-0 space-y-1">
        <h1 className="truncate text-3xl font-semibold tracking-tight text-ink-0">{title}</h1>
        {subtitle && <p className="text-sm text-ink-3">{subtitle}</p>}
      </div>
      <div className="flex w-full flex-wrap items-center gap-2 sm:w-auto">
        {search && (
          <SearchField
            value={search.value}
            onChange={search.onChange}
            aria-label={search.placeholder ?? "Search"}
            className="w-full sm:w-60"
          >
            <SearchField.Group>
              <SearchField.SearchIcon />
              <SearchField.Input placeholder={search.placeholder ?? "Search"} />
              <SearchField.ClearButton />
            </SearchField.Group>
          </SearchField>
        )}
        {actions}
      </div>
    </header>
  );
}
