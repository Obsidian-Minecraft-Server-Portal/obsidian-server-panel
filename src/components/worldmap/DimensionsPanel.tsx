import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { dimensionLabel, type LivePlayer, type WorldMapDimension } from "../../api/worldmap";
import { PanelHeader } from "./PanelHeader";

interface DimensionsPanelProps {
  /** Every dimension the world has, rendered or not. */
  dimensions: readonly WorldMapDimension[];
  players: readonly LivePlayer[];
  /** The folder the map shows now. */
  current: string;
  onChoose: (folder: string) => void;
  onClose: () => void;
}

/** Every dimension in the world, with what is rendered of each. Choosing a rendered one shows it on the map. */
export function DimensionsPanel({ dimensions, players, current, onChoose, onClose }: DimensionsPanelProps) {
  return (
    <aside className="pointer-events-auto flex min-h-0 flex-col overflow-hidden rounded-panel border border-line-strong bg-panel/95 shadow-lift backdrop-blur-md">
      <PanelHeader
        icon="lucide:layers"
        title="Dimensions"
        count={dimensions.length}
        closeLabel="Close the dimensions panel"
        onClose={onClose}
      />
      <ul className="min-h-0 flex-1 overflow-y-auto p-2" data-native-scroll>
        {dimensions.map((dimension) => (
          <DimensionRow
            key={dimension.folder}
            dimension={dimension}
            online={players.filter((player) => player.dimension === dimension.id).length}
            active={dimension.folder === current}
            onPress={() => onChoose(dimension.folder)}
          />
        ))}
      </ul>
    </aside>
  );
}

function DimensionRow({
  dimension,
  online,
  active,
  onPress,
}: {
  dimension: WorldMapDimension;
  online: number;
  active: boolean;
  onPress: () => void;
}) {
  const rendered = dimension.regions > 0;
  return (
    <li>
      <button
        type="button"
        disabled={!rendered}
        aria-current={active ? "true" : undefined}
        onClick={onPress}
        className={cn(
          "flex w-full items-center gap-3 rounded-control px-2 py-2 text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-accent",
          active ? "bg-accent-tint" : "hover:bg-raised",
          !rendered && "cursor-not-allowed opacity-50 hover:bg-transparent",
        )}
      >
        <Icon icon={dimensionIcon(dimension.id)} width={16} height={16} className={active ? "text-accent" : "text-ink-3"} />
        <span className="min-w-0 flex-1">
          <span className="block truncate text-sm font-medium text-ink-0">{dimensionLabel(dimension.id)}</span>
          <span className="block truncate font-mono text-[11px] text-ink-3">{dimension.id}</span>
        </span>
        <span className="shrink-0 text-right font-mono text-[11px] text-ink-3">
          {rendered ? `${dimension.regions} regions` : "Not rendered yet"}
          {online > 0 && ` · ${online} online`}
        </span>
      </button>
    </li>
  );
}

function dimensionIcon(id: string): string {
  switch (id) {
    case "minecraft:overworld":
      return "lucide:trees";
    case "minecraft:the_nether":
      return "lucide:flame";
    case "minecraft:the_end":
      return "lucide:moon";
    default:
      return "lucide:box";
  }
}
