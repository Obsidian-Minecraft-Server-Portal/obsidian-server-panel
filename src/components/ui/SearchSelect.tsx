import { ComboBox, Description, Header, Input, Label, ListBox, cn, useFilter } from "@heroui/react";
import { useMemo, type Key, type ReactNode } from "react";

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
  placeholder,
  description,
  emptyText = "No matches",
  isDisabled,
  isRequired,
  isClearable,
  className,
}: SearchSelectProps) {
  const { contains } = useFilter({ sensitivity: "base" });
  const groups = useMemo(() => groupBySection(options), [options]);
  const sectioned = groups.some(([section]) => section !== "");

  return (
    <ComboBox
      selectedKey={value}
      onSelectionChange={(key: Key | null) => {
        if (key !== null) onChange(String(key));
        else if (isClearable) onChange(null);
      }}
      defaultFilter={contains}
      allowsEmptyCollection
      isDisabled={isDisabled}
      isRequired={isRequired}
      className={cn("flex w-full flex-col gap-1.5", className)}
    >
      <Label className="text-sm font-medium text-ink-2">{label}</Label>
      <ComboBox.InputGroup>
        <Input placeholder={placeholder} autoComplete="off" spellCheck={false} />
        <ComboBox.Trigger />
      </ComboBox.InputGroup>
      {description && <Description className="text-xs text-ink-3">{description}</Description>}
      <ComboBox.Popover className="max-h-80">
        <ListBox renderEmptyState={() => <p className="px-3 py-2 text-sm text-ink-3">{emptyText}</p>}>
          {sectioned
            ? groups.map(([section, items]) => (
                <ListBox.Section key={section || "default"} id={section || "default"}>
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
            : options.map((option) => <OptionItem key={option.id} option={option} />)}
        </ListBox>
      </ComboBox.Popover>
    </ComboBox>
  );
}
