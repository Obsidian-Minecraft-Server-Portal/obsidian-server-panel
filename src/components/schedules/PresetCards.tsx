import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { PRESETS, type Preset } from "./presets";

interface PresetCardsProps {
  onPick: (preset: Preset) => void;
}

export function PresetCards({ onPick }: PresetCardsProps) {
  return (
    <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
      {PRESETS.map((preset) => (
        <Button
          key={preset.id}
          variant="ghost"
          onPress={() => onPick(preset)}
          className="h-auto flex-col items-start gap-1.5 whitespace-normal rounded-panel border border-line bg-raised/30 p-4 text-left hover:border-line-strong"
        >
          <Icon icon={preset.icon} width={20} height={20} className="text-secondary" />
          <span className="text-sm font-semibold text-ink-0">{preset.title}</span>
          <span className="text-xs text-ink-3">{preset.description}</span>
        </Button>
      ))}
    </div>
  );
}
