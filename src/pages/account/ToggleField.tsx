import { Switch } from "@heroui/react";

interface ToggleFieldProps {
  label: string;
  description?: string;
  value: boolean;
  onChange: (value: boolean) => void;
  isDisabled?: boolean;
}

export function ToggleField({ label, description, value, onChange, isDisabled }: ToggleFieldProps) {
  return (
    <Switch isSelected={value} onChange={onChange} isDisabled={isDisabled} className="flex items-center justify-between gap-4">
      <Switch.Content>
        <span className="block text-sm font-medium text-ink-1">{label}</span>
        {description && <span className="block text-xs text-ink-3">{description}</span>}
      </Switch.Content>
      <Switch.Control>
        <Switch.Thumb />
      </Switch.Control>
    </Switch>
  );
}
