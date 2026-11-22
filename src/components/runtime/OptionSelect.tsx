import { Label, ListBox, Select } from "@heroui/react";
import type { Key } from "react";

export interface SelectOption {
  id: string;
  label: string;
}

interface OptionSelectProps {
  label: string;
  options: SelectOption[];
  value: string | null;
  onChange: (value: string | null) => void;
  placeholder?: string;
  isDisabled?: boolean;
  className?: string;
}

export function OptionSelect({ label, options, value, onChange, placeholder, isDisabled, className }: OptionSelectProps) {
  return (
    <Select
      value={value}
      onChange={(key: Key | null) => onChange(key === null ? null : String(key))}
      placeholder={placeholder}
      isDisabled={isDisabled}
      className={className}
    >
      <Label className="text-sm font-medium text-ink-2">{label}</Label>
      <Select.Trigger>
        <Select.Value />
        <Select.Indicator />
      </Select.Trigger>
      <Select.Popover>
        <ListBox>
          {options.map((option) => (
            <ListBox.Item key={option.id} id={option.id} textValue={option.label}>
              {option.label}
              <ListBox.ItemIndicator />
            </ListBox.Item>
          ))}
        </ListBox>
      </Select.Popover>
    </Select>
  );
}
