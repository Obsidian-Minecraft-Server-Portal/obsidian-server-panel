import { Icon } from "@iconify-icon/react";
import { Button, cn } from "@heroui/react";
import type { ViewMode } from "./useViewMode";

const OPTIONS: { mode: ViewMode; icon: string; label: string }[] = [
  { mode: "grid", icon: "lucide:layout-grid", label: "Grid view" },
  { mode: "compact", icon: "lucide:list", label: "Compact view" },
  { mode: "table", icon: "lucide:table-2", label: "Table view" },
];

interface ViewModeSwitchProps {
  value: ViewMode;
  onChange: (mode: ViewMode) => void;
}

export function ViewModeSwitch({ value, onChange }: ViewModeSwitchProps) {
  return (
    <div role="group" aria-label="Instance view" className="flex rounded-control border border-line bg-input p-1">
      {OPTIONS.map(({ mode, icon, label }) => (
        <Button
          key={mode}
          type="button"
          aria-label={label}
          aria-pressed={value === mode}
          onPress={() => onChange(mode)}
          className={cn(
            "grid size-8 place-items-center rounded-[9px] transition-colors",
            value === mode ? "bg-accent-tint text-accent" : "text-ink-3 hover:text-ink-1",
          )}
        >
          <Icon icon={icon} width={16} height={16} />
        </Button>
      ))}
    </div>
  );
}
