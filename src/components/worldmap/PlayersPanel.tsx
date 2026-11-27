import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { dimensionLabel, playersSummary, type LivePlayer, type WorldMapPlayers } from "../../api/worldmap";
import { Tip } from "../ui/Tip";
import type { FollowMode } from "./map-focus";
import { PanelHeader } from "./PanelHeader";
import { PlayerFace } from "./PlayerTag";

/** The player the views follow, and whether they are followed from behind or from inside their head. */
export interface Following {
  name: string;
  mode: FollowMode;
}

interface PlayersPanelProps {
  live: WorldMapPlayers | undefined;
  dimensionId: string;
  skinBase: string;
  focused: string | null;
  following: Following | null;
  onClose: () => void;
  onFocus: (player: LivePlayer) => void;
  onFollow: (player: LivePlayer, mode: FollowMode) => void;
}

/**
 * Who is online, in a panel that slides over the map. A click looks at a player. The locate button follows them from behind,
 * and the eye button puts the view inside their head.
 */
export function PlayersPanel({ live, dimensionId, skinBase, focused, following, onClose, onFocus, onFollow }: PlayersPanelProps) {
  const players = live?.players ?? [];
  return (
    <aside className="pointer-events-auto flex min-h-0 flex-1 flex-col overflow-hidden rounded-panel border border-line-strong bg-panel/95 shadow-lift backdrop-blur-md">
      <PanelHeader icon="lucide:users" title="Players" count={players.length} closeLabel="Close the players panel" onClose={onClose} />
      {players.length === 0 ? (
        <p className={cn("px-4 py-6 text-center text-sm", live?.state === "unavailable" ? "text-rose" : "text-ink-3")}>
          {live?.state === "live" ? "Nobody is online." : playersSummary(live)}
        </p>
      ) : (
        <ul className="min-h-0 flex-1 overflow-y-auto p-2" data-native-scroll>
          {players.map((player) => (
            <PlayerRow
              key={player.name}
              player={player}
              elsewhere={player.dimension !== dimensionId}
              skinBase={skinBase}
              focused={focused === player.name}
              mode={following?.name === player.name ? following.mode : null}
              onFocus={() => onFocus(player)}
              onFollow={(mode) => onFollow(player, mode)}
            />
          ))}
        </ul>
      )}
      {live?.state === "live" && players.length > 0 && (
        <p className="border-t border-line px-4 py-2 font-mono text-[11px] text-ink-3">{playersSummary(live)}</p>
      )}
    </aside>
  );
}

interface PlayerRowProps {
  player: LivePlayer;
  elsewhere: boolean;
  skinBase: string;
  focused: boolean;
  /** How this player is followed right now, if they are. */
  mode: FollowMode | null;
  onFocus: () => void;
  onFollow: (mode: FollowMode) => void;
}

function PlayerRow({ player, elsewhere, skinBase, focused, mode, onFocus, onFollow }: PlayerRowProps) {
  const [x, y, z] = player.position.map(Math.floor);
  return (
    <li
      className={cn(
        "group flex items-center gap-1 rounded-control transition-colors",
        mode ? "bg-accent-tint" : focused ? "bg-raised" : "hover:bg-raised",
      )}
    >
      <button
        type="button"
        onClick={onFocus}
        className="flex min-w-0 flex-1 items-center gap-3 rounded-control px-2 py-2 text-left outline-none focus-visible:ring-2 focus-visible:ring-accent"
      >
        <PlayerFace src={skinBase + player.skin} />
        <span className="min-w-0 flex-1">
          <span className="block truncate text-sm font-medium text-ink-0">{player.name}</span>
          <span className="block truncate font-mono text-[11px] text-ink-3">
            {elsewhere ? dimensionLabel(player.dimension) : `${x}, ${y}, ${z}`}
          </span>
        </span>
      </button>
      <ModeButton
        label={`Follow ${player.name} from behind`}
        tip="Follow from behind"
        icon={mode === "behind" ? "lucide:locate-fixed" : "lucide:locate"}
        active={mode === "behind"}
        onPress={() => onFollow("behind")}
      />
      <ModeButton
        label={`Look out of ${player.name}'s head`}
        tip="Look from inside their head"
        icon="lucide:eye"
        active={mode === "head"}
        onPress={() => onFollow("head")}
      />
    </li>
  );
}

function ModeButton({
  label,
  tip,
  icon,
  active,
  onPress,
}: {
  label: string;
  tip: string;
  icon: string;
  active: boolean;
  onPress: () => void;
}) {
  return (
    <Tip label={active ? "Stop" : tip} placement="left">
      <button
        type="button"
        aria-label={label}
        aria-pressed={active}
        onClick={onPress}
        className={cn(
          "grid size-8 shrink-0 place-items-center rounded-[9px] transition-colors outline-none focus-visible:ring-2 focus-visible:ring-accent",
          active ? "text-accent" : "text-ink-3 hover:bg-input hover:text-ink-0",
        )}
      >
        <Icon icon={icon} width={16} height={16} />
      </button>
    </Tip>
  );
}
