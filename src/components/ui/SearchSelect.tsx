import { ComboBox, Description, Header, Input, Label, ListBox, cn } from "@heroui/react";
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
  placeholder?: string;
  description?: ReactNode;
  emptyText?: string;
  isDisabled?: boolean;
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
  placeholder,
  description,
  emptyText = "No matches",
  isDisabled,
  className,
}: SearchSelectProps) {
  const selected = options.find((option) => option.id === value);
  const [query, setQuery] = useState<string | null>(null);
  const inputValue = query ?? selected?.label ?? "";

  const filtered = useMemo(() => {
    const needle = (query ?? "").trim().toLowerCase();
    if (!needle || needle === selected?.label.toLowerCase()) return options;
    return options.filter((option) =>
      `${option.label} ${option.detail ?? ""} ${option.section ?? ""}`.toLowerCase().includes(needle),
    );
  }, [options, query, selected]);
  const groups = useMemo(() => groupBySection(filtered), [filtered]);
  const sectioned = groups.some(([section]) => section !== "");

  return (
    <ComboBox
      selectedKey={value}
      inputValue={inputValue}
      onInputChange={setQuery}
      onSelectionChange={(key: Key | null) => {
        setQuery(null);
        if (key !== null) onChange(String(key));
      }}
      onOpenChange={(open) => {
        if (!open) setQuery(null);
      }}
      menuTrigger="focus"
      allowsEmptyCollection
      isDisabled={isDisabled}
      className={cn("flex w-full flex-col gap-1.5", className)}
    >
      <Label className="text-sm font-medium text-ink-2">{label}</Label>
      <ComboBox.InputGroup>
        <Input placeholder={placeholder} />
        <ComboBox.Trigger />
      </ComboBox.InputGroup>
      {description && <Description className="text-xs text-ink-3">{description}</Description>}
      <ComboBox.Popover className="max-h-80">
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
      </ComboBox.Popover>
    </ComboBox>
  );
}
