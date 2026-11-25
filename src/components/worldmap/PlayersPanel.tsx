import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { dimensionLabel, playersSummary, type LivePlayer, type WorldMapPlayers } from "../../api/worldmap";
import { Tip } from "../ui/Tip";
import { PlayerFace } from "./PlayerTag";

interface PlayersPanelProps {
  live: WorldMapPlayers | undefined;
  dimensionId: string;
  skinBase: string;
  focused: string | null;
  following: string | null;
  onFocus: (player: LivePlayer) => void;
  onFollow: (player: LivePlayer) => void;
}

/** Who is online, with a click to look at a player and a toggle to keep the view on them. */
export function PlayersPanel({ live, dimensionId, skinBase, focused, following, onFocus, onFollow }: PlayersPanelProps) {
  const players = live?.players ?? [];
  return (
    <aside className="flex min-h-0 flex-col overflow-hidden rounded-panel border border-line bg-panel lg:w-72">
      <header className="flex items-center justify-between gap-2 border-b border-line px-4 py-3">
        <div className="flex items-center gap-2">
          <Icon icon="lucide:users" width={15} height={15} className="text-ink-3" />
          <h3 className="text-sm font-semibold text-ink-0">Players</h3>
        </div>
        <span className="rounded-full bg-input px-2 py-0.5 font-mono text-[11px] text-ink-2">{players.length}</span>
      </header>
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
              following={following === player.name}
              onFocus={() => onFocus(player)}
              onFollow={() => onFollow(player)}
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
  following: boolean;
  onFocus: () => void;
  onFollow: () => void;
}

function PlayerRow({ player, elsewhere, skinBase, focused, following, onFocus, onFollow }: PlayerRowProps) {
  const [x, y, z] = player.position.map(Math.floor);
  return (
    <li
      className={cn(
        "group flex items-center gap-1 rounded-control transition-colors",
        following ? "bg-accent-tint" : focused ? "bg-raised" : "hover:bg-raised",
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
      <Tip label={following ? "Stop following" : "Follow"} placement="left">
        <button
          type="button"
          aria-label={following ? `Stop following ${player.name}` : `Follow ${player.name}`}
          aria-pressed={following}
          onClick={onFollow}
          className={cn(
            "mr-1 grid size-8 shrink-0 place-items-center rounded-[9px] transition-colors outline-none focus-visible:ring-2 focus-visible:ring-accent",
            following ? "text-accent" : "text-ink-3 hover:bg-input hover:text-ink-0",
          )}
        >
          <Icon icon={following ? "lucide:locate-fixed" : "lucide:locate"} width={16} height={16} />
        </button>
      </Tip>
    </li>
  );
}
