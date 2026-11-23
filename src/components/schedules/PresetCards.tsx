import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { PRESETS, type Preset } from "./presets";

interface PresetCardsProps {
  onPick: (preset: Preset) => void;
}

export function PresetCards({ onPick }: PresetCardsProps) {
  return (
    <div className="grid w-full gap-3 sm:grid-cols-2 xl:grid-cols-5">
      {PRESETS.map((preset, index) => (
        <Button
          key={preset.id}
          variant="ghost"
          onPress={() => onPick(preset)}
          className={cn(
            "h-auto w-full flex-col items-start gap-1.5 whitespace-normal rounded-panel border border-line bg-raised/30 p-4 text-left hover:border-line-strong",
            index === PRESETS.length - 1 && "sm:col-span-2 xl:col-span-1",
          )}
        >
          <Icon icon={preset.icon} width={20} height={20} className="text-secondary" />
          <span className="text-sm font-semibold text-ink-0">{preset.title}</span>
          <span className="text-xs text-ink-3">{preset.description}</span>
        </Button>
      ))}
    </div>
  );
}
