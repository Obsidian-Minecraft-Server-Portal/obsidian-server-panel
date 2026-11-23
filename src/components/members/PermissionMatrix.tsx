import { useMemo } from "react";
import { CheckboxRow } from "../ui/CheckboxRow";
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
    <div className="columns-1 gap-3 sm:columns-2 xl:columns-3">
      {groups.map((group) => (
        <section key={group.key} className="mb-3 break-inside-avoid rounded-control border border-line bg-input/50 px-4 py-3">
          <h4 className="mb-1 font-mono text-[10px] font-medium tracking-widest text-ink-3 uppercase">{group.label}</h4>
          <ul className="divide-y divide-line">
            {group.permissions.map((name) => (
              <li key={name} className="py-2.5">
                <CheckboxRow
                  isSelected={selected.has(name)}
                  isDisabled={disabled}
                  onChange={(on) => toggle(name, on)}
                >
                  {permissionLabel(name)}
                </CheckboxRow>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
