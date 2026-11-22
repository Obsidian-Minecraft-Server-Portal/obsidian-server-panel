import { Checkbox, cn } from "@heroui/react";
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
    <div className="grid items-start gap-3 sm:grid-cols-2 xl:grid-cols-3">
      {groups.map((group) => (
        <section key={group.key} className="rounded-control border border-line bg-input/50 px-4 py-3">
          <h4 className="mb-1 font-mono text-[10px] font-medium tracking-widest text-ink-3 uppercase">{group.label}</h4>
          <ul className="divide-y divide-line">
            {group.permissions.map((name) => (
              <li key={name}>
                <Checkbox
                  isSelected={selected.has(name)}
                  isDisabled={disabled}
                  onChange={(on: boolean) => toggle(name, on)}
                  className={cn("!flex !flex-row !items-center gap-3 py-2.5", disabled ? "opacity-60" : "cursor-pointer")}
                >
                  <Checkbox.Control className="size-4 shrink-0 rounded-[5px] border border-ink-4 bg-input data-[selected=true]:border-accent data-[selected=true]:bg-accent">
                    <Checkbox.Indicator />
                  </Checkbox.Control>
                  <Checkbox.Content className="text-sm text-ink-1">{permissionLabel(name)}</Checkbox.Content>
                </Checkbox>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
