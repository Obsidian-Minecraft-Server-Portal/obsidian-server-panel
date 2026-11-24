import { ComboBox, Input, Label, ListBox } from "@heroui/react";

interface UsernameFieldProps {
  value: string;
  onChange: (value: string) => void;
  suggestions: string[];
  emptyText: string;
  isDisabled?: boolean;
}

export function UsernameField({ value, onChange, suggestions, emptyText, isDisabled }: UsernameFieldProps) {
  const items = suggestions.map((name) => ({ id: name, name }));

  return (
    <ComboBox
      inputValue={value}
      onInputChange={onChange}
      onSelectionChange={(key) => key !== null && onChange(String(key))}
      items={items}
      allowsCustomValue
      allowsEmptyCollection
      menuTrigger="input"
      isRequired
      isDisabled={isDisabled}
      className="flex min-w-56 flex-1 flex-col gap-1.5"
    >
      <Label className="text-sm font-medium text-ink-2">Panel username</Label>
      <ComboBox.InputGroup>
        <Input placeholder="Type a username" autoComplete="off" spellCheck={false} />
        <ComboBox.Trigger />
      </ComboBox.InputGroup>
      <ComboBox.Popover className="max-h-72">
        <ListBox renderEmptyState={() => <p className="px-3 py-2 text-sm text-ink-3">{emptyText}</p>}>
          {(item: { id: string; name: string }) => (
            <ListBox.Item id={item.id} textValue={item.name}>
              {item.name}
              <ListBox.ItemIndicator />
            </ListBox.Item>
          )}
        </ListBox>
      </ComboBox.Popover>
    </ComboBox>
  );
}
