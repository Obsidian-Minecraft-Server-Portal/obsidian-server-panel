import { ToggleButton, ToggleButtonGroup } from "@heroui/react";
import { Icon } from "@iconify-icon/react";

export type ViewMode = "grid" | "compact" | "table";

const VIEWS: { id: ViewMode; icon: string; label: string }[] = [
  { id: "grid", icon: "lucide:layout-grid", label: "Grid view" },
  { id: "compact", icon: "lucide:list", label: "Compact view" },
  { id: "table", icon: "lucide:table-2", label: "Table view" },
];

interface ViewToggleProps {
  value: ViewMode;
  onChange: (mode: ViewMode) => void;
  label?: string;
}

export function ViewToggle({ value, onChange, label = "View" }: ViewToggleProps) {
  return (
    <ToggleButtonGroup
      aria-label={label}
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={new Set([value])}
      onSelectionChange={(keys) => {
        const next = [...keys][0];
        if (next !== undefined) onChange(String(next) as ViewMode);
      }}
      className="h-10 gap-0.5 rounded-control border border-line bg-input p-1"
    >
      {VIEWS.map((entry) => (
        <ToggleButton
          key={entry.id}
          id={entry.id}
          variant="ghost"
          isIconOnly
          aria-label={entry.label}
          className="size-8 min-w-0 rounded-[9px] bg-transparent text-ink-3 transition-colors hover:bg-raised hover:text-ink-1 data-[selected=true]:bg-raised data-[selected=true]:text-ink-0 data-[selected=true]:shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]"
        >
          <Icon icon={entry.icon} width={16} height={16} />
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
