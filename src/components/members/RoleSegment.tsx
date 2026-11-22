import { Button, cn } from "@heroui/react";
import { roleLabel } from "./format";

interface RoleSegmentProps {
  roles: string[];
  value: string;
  onChange: (role: string) => void;
  disabled?: boolean;
}

export function RoleSegment({ roles, value, onChange, disabled }: RoleSegmentProps) {
  return (
    <div className="inline-flex flex-wrap gap-1 rounded-control border border-line bg-input p-1">
      {roles.map((role) => {
        const active = role === value;
        return (
          <Button
            key={role}
            size="sm"
            variant="ghost"
            isDisabled={disabled}
            onPress={() => onChange(role)}
            className={cn(
              "rounded-[9px] font-medium",
              active ? "bg-raised text-ink-0" : "text-ink-3 hover:text-ink-1",
            )}
          >
            {roleLabel(role)}
          </Button>
        );
      })}
    </div>
  );
}
