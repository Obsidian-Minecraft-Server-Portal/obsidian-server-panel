import { Autocomplete, Description, Header, Label, ListBox, SearchField, cn } from "@heroui/react";
import { useMemo, useState, type Key, type ReactNode } from "react";

export interface SearchOption {
  id: string;
  label: string;
  detail?: string;
  section?: string;
}

interface SearchSelectProps {
  label: string;
  options: SearchOption[];
  value: string | null;
  onChange: (value: string | null) => void;
  onSearch?: (text: string) => void;
  placeholder?: string;
  description?: ReactNode;
  emptyText?: string;
  isDisabled?: boolean;
  isRequired?: boolean;
  isClearable?: boolean;
  className?: string;
}

function groupBySection(options: SearchOption[]): [string, SearchOption[]][] {
  const groups = new Map<string, SearchOption[]>();
  for (const option of options) {
    const key = option.section ?? "";
    groups.set(key, [...(groups.get(key) ?? []), option]);
  }
  return [...groups];
}

function OptionItem({ option }: { option: SearchOption }) {
  return (
    <ListBox.Item id={option.id} textValue={option.label} className="flex items-center gap-3">
      <span className="min-w-0 flex-1 truncate">{option.label}</span>
      {option.detail && <span className="shrink-0 font-mono text-[11px] tracking-wide text-ink-3 uppercase">{option.detail}</span>}
      <ListBox.ItemIndicator />
    </ListBox.Item>
  );
}

export function SearchSelect({
  label,
  options,
  value,
  onChange,
  onSearch,
  placeholder,
  description,
  emptyText = "No matches",
  isDisabled,
  isRequired,
  isClearable,
  className,
}: SearchSelectProps) {
  const [query, setQuery] = useState("");
  const selected = options.find((option) => option.id === value);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return options;
    return options.filter((option) =>
      `${option.label} ${option.detail ?? ""} ${option.section ?? ""}`.toLowerCase().includes(needle),
    );
  }, [options, query]);
  const groups = useMemo(() => groupBySection(filtered), [filtered]);
  const sectioned = groups.some(([section]) => section !== "");

  return (
    <Autocomplete
      selectedKey={value}
      onSelectionChange={(key: Key | null) => {
        setQuery("");
        if (key !== null) onChange(String(key));
      }}
      onOpenChange={(open) => {
        if (!open) setQuery("");
      }}
      onClear={isClearable ? () => onChange(null) : undefined}
      isDisabled={isDisabled}
      isRequired={isRequired}
      className={cn("flex w-full flex-col gap-1.5", className)}
    >
      <Label className="text-sm font-medium text-ink-2">
        {label}
      </Label>
      <Autocomplete.Trigger>
        <Autocomplete.Value>
          {value === null ? (
            <span className="truncate text-ink-4">{placeholder}</span>
          ) : (
            <span className="truncate">{selected?.label ?? ""}</span>
          )}
        </Autocomplete.Value>
        {isClearable && value !== null && <Autocomplete.ClearButton />}
        <Autocomplete.Indicator />
      </Autocomplete.Trigger>
      {description && <Description className="text-xs text-ink-3">{description}</Description>}
      <Autocomplete.Popover className="max-h-80">
        <Autocomplete.Filter
          inputValue={query}
          onInputChange={(text: string) => {
            setQuery(text);
            onSearch?.(text);
          }}
        >
          <SearchField autoFocus aria-label={`Search ${label}`} className="p-2">
            <SearchField.Group>
              <SearchField.SearchIcon />
              <SearchField.Input placeholder={placeholder ?? "Search"} />
              <SearchField.ClearButton />
            </SearchField.Group>
          </SearchField>
          <ListBox renderEmptyState={() => <p className="px-3 py-2 text-sm text-ink-3">{emptyText}</p>}>
            {sectioned
              ? groups.map(([section, items]) => (
                  <ListBox.Section key={section || "default"}>
                    {section && (
                      <Header className="px-3 pt-2 pb-1 font-mono text-[10px] tracking-widest text-ink-4 uppercase">
                        {section}
                      </Header>
                    )}
                    {items.map((option) => (
                      <OptionItem key={option.id} option={option} />
                    ))}
                  </ListBox.Section>
                ))
              : filtered.map((option) => <OptionItem key={option.id} option={option} />)}
          </ListBox>
        </Autocomplete.Filter>
      </Autocomplete.Popover>
    </Autocomplete>
  );
}
