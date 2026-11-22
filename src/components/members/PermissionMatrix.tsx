import { Checkbox } from "@heroui/react";
import { useMemo } from "react";
import { groupPermissions, permissionLabel } from "./format";

interface PermissionMatrixProps {
  catalog: string[];
  value: string[];
  onChange: (next: string[]) => void;
  disabled?: boolean;
}

export function PermissionMatrix({ catalog, value, onChange, disabled }: PermissionMatrixProps) {
  const groups = useMemo(() => groupPermissions(catalog), [catalog]);
  const selected = useMemo(() => new Set(value), [value]);

  const toggle = (name: string, on: boolean) => {
    const next = new Set(selected);
    if (on) next.add(name);
    else next.delete(name);
    onChange(catalog.filter((permission) => next.has(permission)));
  };

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      {groups.map((group) => (
        <div key={group.key} className="rounded-control border border-line bg-input/60 p-4">
          <h4 className="mb-3 font-mono text-[11px] font-medium tracking-wider text-ink-3 uppercase">{group.label}</h4>
          <div className="space-y-2.5">
            {group.permissions.map((name) => (
              <Checkbox
                key={name}
                isSelected={selected.has(name)}
                isDisabled={disabled}
                onChange={(on: boolean) => toggle(name, on)}
              >
                <Checkbox.Control>
                  <Checkbox.Indicator />
                </Checkbox.Control>
                <Checkbox.Content className="text-sm text-ink-1">{permissionLabel(name)}</Checkbox.Content>
              </Checkbox>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
