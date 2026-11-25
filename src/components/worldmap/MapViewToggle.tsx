import { ToggleButton, ToggleButtonGroup } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { Tip } from "../ui/Tip";
import type { MapView } from "./map-memory";

const VIEWS: { id: MapView; icon: string; label: string }[] = [
  { id: "top", icon: "lucide:map", label: "Top-down" },
  { id: "free", icon: "lucide:box", label: "3D" },
];

/** Switches between the top-down map and the 3D view. It floats over the map, so it sits on a glass backdrop. */
export function MapViewToggle({ value, onChange }: { value: MapView; onChange: (view: MapView) => void }) {
  return (
    <ToggleButtonGroup
      aria-label="Map view"
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={new Set([value])}
      onSelectionChange={(keys) => {
        const next = [...keys][0];
        if (next !== undefined) onChange(String(next) as MapView);
      }}
      className="gap-0.5 rounded-control border border-line-strong bg-canvas/80 p-1 shadow-lift backdrop-blur-md"
    >
      {VIEWS.map((entry) => (
        <Tip key={entry.id} label={entry.label} placement="bottom">
          <ToggleButton
            id={entry.id}
            variant="ghost"
            isIconOnly
            aria-label={entry.label}
            className="size-8 min-w-0 rounded-[9px] bg-transparent text-ink-2 transition-colors hover:bg-raised hover:text-ink-0 data-[selected=true]:bg-accent-tint data-[selected=true]:text-accent"
          >
            <Icon icon={entry.icon} width={16} height={16} />
          </ToggleButton>
        </Tip>
      ))}
    </ToggleButtonGroup>
  );
}
