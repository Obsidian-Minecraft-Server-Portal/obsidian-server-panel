import { cn } from "@heroui/react";
import type { Player, PlayerAction } from "../../../api/instances";
import { Chip } from "../../ui/Chip";
import { GhostButton } from "../../ui/GhostButton";

interface PlayerRowProps {
  player: Player;
  disabled: boolean;
  onAction: (action: PlayerAction) => void;
}

export function PlayerRow({ player, disabled, onAction }: PlayerRowProps) {
  return (
    <div className="flex flex-wrap items-center gap-x-6 gap-y-3 border-t border-line py-4 first:border-t-0 first:pt-0">
      <div className="flex min-w-48 flex-1 items-center gap-3">
        <span
          className={cn(
            "grid size-9 shrink-0 place-items-center rounded-control font-mono text-sm font-semibold uppercase",
            player.online ? "bg-green/12 text-green" : "bg-raised text-ink-3",
          )}
        >
          {player.name.charAt(0)}
        </span>
        <div className="min-w-0">
          <p className="truncate font-semibold text-ink-0">{player.name}</p>
          <p className="truncate font-mono text-[11px] text-ink-4" title={player.uuid ?? undefined}>
            {player.uuid ?? "UUID unknown"}
          </p>
        </div>
      </div>
      <div className="flex flex-wrap gap-2">
        {player.online && <Chip tone="green" pulse>Online</Chip>}
        {player.op && <Chip tone="secondary">Op</Chip>}
        {player.whitelisted && <Chip>Whitelisted</Chip>}
        {player.banned && <Chip tone="rose">Banned</Chip>}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <GhostButton size="sm" isDisabled={disabled} onPress={() => onAction(player.op ? "deop" : "op")}>
          {player.op ? "Deop" : "Op"}
        </GhostButton>
        <GhostButton size="sm" isDisabled={disabled} onPress={() => onAction(player.whitelisted ? "unwhitelist" : "whitelist")}>
          {player.whitelisted ? "Unwhitelist" : "Whitelist"}
        </GhostButton>
        {player.online && (
          <GhostButton size="sm" isDisabled={disabled} onPress={() => onAction("kick")}>
            Kick
          </GhostButton>
        )}
        <GhostButton
          size="sm"
          isDisabled={disabled}
          onPress={() => onAction(player.banned ? "pardon" : "ban")}
          className={player.banned ? undefined : "text-rose"}
        >
          {player.banned ? "Pardon" : "Ban"}
        </GhostButton>
      </div>
    </div>
  );
}
