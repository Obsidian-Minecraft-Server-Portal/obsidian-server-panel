import { useMemo, useState } from "react";
import { hasPermission, useInstance, usePlayerAction, usePlayers, type Player } from "../../../api/instances";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { AddPlayerForm } from "../../../components/instance/players/AddPlayerForm";
import { PlayerRow } from "../../../components/instance/players/PlayerRow";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Panel } from "../../../components/ui/Panel";

function byOnlineThenName(a: Player, b: Player): number {
  return Number(b.online) - Number(a.online) || a.name.localeCompare(b.name);
}

export function PlayersPanels() {
  const id = useInstanceId();
  const players = usePlayers(id);
  const action = usePlayerAction(id);
  const instance = useInstance(id);
  const canManage = instance.data ? hasPermission(instance.data, "players.manage") : false;
  const [search, setSearch] = useState("");

  const visible = useMemo(() => {
    const query = search.trim().toLowerCase();
    return (players.data ?? []).filter((player) => player.name.toLowerCase().includes(query)).sort(byOnlineThenName);
  }, [players.data, search]);

  const error = players.error ?? action.error;

  return (
    <div className="flex h-full min-h-0 flex-col gap-5">
      <Panel
        className="flex min-h-fit flex-1 flex-col"
        title={`Players${players.data ? ` · ${players.data.length}` : ""}`}
        actions={
          <input
            type="search"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Filter"
            aria-label="Filter players"
            className="h-9 w-32 rounded-control border border-line bg-input px-3 text-sm text-ink-1 placeholder:text-ink-4 focus:border-accent"
          />
        }
      >
        {error && <p className="mb-4 text-sm text-rose">{error.message}</p>}
        {players.isPending && <p className="text-sm text-ink-3">Loading players…</p>}
        <div className="flex min-h-fit flex-1 flex-col">
          {players.data && visible.length === 0 && (
            <EmptyState
              className="flex-1 py-8"
              icon="lucide:users"
              title="No players yet"
              description="Players appear once they join, or when you add them by name."
            />
          )}
          {visible.map((player) => (
            <PlayerRow
              key={player.name}
              player={player}
              disabled={!canManage || action.isPending}
              onAction={(next) => action.mutate({ name: player.name, action: next })}
            />
          ))}
        </div>
      </Panel>
      {canManage && (
        <Panel title="Add player">
          <AddPlayerForm disabled={action.isPending} onAction={(name, next) => action.mutate({ name, action: next })} />
          <p className="mt-4 text-xs text-ink-3">
            Changes apply immediately while the server runs, and edit the JSON lists while it is stopped.
          </p>
        </Panel>
      )}
    </div>
  );
}
