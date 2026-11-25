import { useCallback, useMemo, useRef, useState } from "react";
import {
  dimensionLabel,
  renderSummary,
  useWorldMapPlayers,
  useWorldMapStatus,
  worldMapSkinBase,
  worldMapSource,
  type LivePlayer,
  type WorldMapStatus,
} from "../../api/worldmap";
import { EmptyState } from "../ui/EmptyState";
import { SectionTabs } from "../ui/SectionTabs";
import type { MapFocus } from "./map-focus";
import { mapMemory, rememberView, type MapView } from "./map-memory";
import { MapViewToggle } from "./MapViewToggle";
import { PlayersPanel } from "./PlayersPanel";
import { TileView } from "./TileView";
import { WorldMapViewer } from "./WorldMapViewer";

const NO_PLAYERS: readonly LivePlayer[] = [];

/** The built-in map: the top-down tiles or the free-fly 3D view, a dimension picker, the render status and the players. */
export function WorldMapPanel({ instanceId, canViewPlayers }: { instanceId: number; canViewPlayers: boolean }) {
  const status = useWorldMapStatus(instanceId);
  const live = useWorldMapPlayers(instanceId, canViewPlayers);
  const players = live.data?.players ?? NO_PLAYERS;
  const [picked, setPicked] = useState<string | null>(() => mapMemory(instanceId).folder ?? null);
  const [view, setViewState] = useState<MapView>(() => mapMemory(instanceId).view ?? "top");
  const [request, setRequest] = useState<{ name: string; nonce: number } | null>(null);
  const [following, setFollowing] = useState<string | null>(null);
  const nonce = useRef(0);
  const guardWheel = useCallback(keepWheelInside, []);

  const dimensions = status.data?.dimensions.filter((dimension) => dimension.regions > 0) ?? [];
  const dimension = dimensions.find((entry) => entry.folder === picked) ?? dimensions[0];
  const target = players.find((player) => player.name === request?.name);
  const onScreen = target !== undefined && target.dimension === dimension?.id;
  const [x, y, z] = target?.position ?? [0, 0, 0];
  const focus = useMemo<MapFocus | null>(
    () =>
      onScreen && request
        ? { x, y, z, yaw: target.yaw, nonce: request.nonce, follow: following === request.name }
        : null,
    // The player object changes on every reading; the focus only changes when the numbers do.
    [onScreen, request, following, x, y, z],
  );

  if (status.isPending) return <p className="text-sm text-ink-3">Loading world map…</p>;
  if (status.isError) return <p className="text-sm text-rose">{status.error.message}</p>;
  if (!dimension) {
    return (
      <EmptyState
        icon="lucide:map"
        title={status.data.rendering ? "Rendering the first map" : "Nothing rendered yet"}
        description={
          status.data.rendering
            ? "Large worlds take a while. The map appears here once the first regions are done."
            : "The map renders from the world files. Turn updates on in Settings, or render now."
        }
      />
    );
  }

  const chooseDimension = (folder: string) => {
    setPicked(folder);
    rememberView(instanceId, { folder });
  };
  const chooseView = (next: MapView) => {
    setViewState(next);
    rememberView(instanceId, { view: next });
  };
  const focusPlayer = (player: LivePlayer) => {
    const home = dimensions.find((entry) => entry.id === player.dimension);
    if (home && home.folder !== dimension.folder) chooseDimension(home.folder);
    nonce.current += 1;
    setRequest({ name: player.name, nonce: nonce.current });
  };
  const followPlayer = (player: LivePlayer) => {
    if (following === player.name) {
      setFollowing(null);
      return;
    }
    setFollowing(player.name);
    focusPlayer(player);
  };
  const tabs = dimensions.map((entry) => ({ id: entry.folder, label: dimensionLabel(entry.id) }));

  return (
    <div className="flex min-h-[32rem] flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        {tabs.length > 1 ? (
          <SectionTabs label="Dimensions" tabs={tabs} value={dimension.folder} onChange={chooseDimension} />
        ) : (
          <p className="font-mono text-xs text-ink-3 uppercase">{dimensionLabel(dimension.id)}</p>
        )}
        <RenderLine status={status.data} />
      </div>
      <div className="flex min-h-0 flex-col gap-3 lg:h-[72vh] lg:min-h-[32rem] lg:flex-row">
        <div
          ref={guardWheel}
          data-native-scroll
          className="relative h-[65vh] min-h-[24rem] flex-1 overflow-hidden rounded-panel border border-line bg-canvas lg:h-auto"
        >
          {view === "top" ? (
            <TileView
              key={dimension.folder}
              instanceId={instanceId}
              folder={dimension.folder}
              dimensionId={dimension.id}
              players={players}
              revision={status.data.revision}
              focus={focus}
              onPan={() => setFollowing(null)}
            />
          ) : (
            <FreeView
              key={dimension.folder}
              instanceId={instanceId}
              folder={dimension.folder}
              dimensionId={dimension.id}
              players={players}
              revision={status.data.revision}
              focus={focus}
            />
          )}
          <div className="absolute top-3 left-3">
            <MapViewToggle value={view} onChange={chooseView} />
          </div>
        </div>
        {canViewPlayers && (
          <PlayersPanel
            live={live.data}
            dimensionId={dimension.id}
            skinBase={worldMapSkinBase(instanceId)}
            focused={request?.name ?? null}
            following={following}
            onFocus={focusPlayer}
            onFollow={followPlayer}
          />
        )}
      </div>
    </div>
  );
}

function RenderLine({ status }: { status: WorldMapStatus }) {
  return (
    <p className="flex items-center gap-2 font-mono text-xs text-ink-3">
      {status.rendering && <span className="size-1.5 animate-pulse rounded-full bg-accent" />}
      {status.regions} regions · {renderSummary(status)}
    </p>
  );
}

/** Wheel input over the map zooms or sets the flying speed, so the page around it must not scroll as well. */
function keepWheelInside(node: HTMLDivElement | null) {
  if (!node) return undefined;
  const stop = (event: WheelEvent) => event.preventDefault();
  node.addEventListener("wheel", stop, { passive: false });
  return () => node.removeEventListener("wheel", stop);
}

interface FreeViewProps {
  instanceId: number;
  folder: string;
  dimensionId: string;
  players: readonly LivePlayer[];
  revision: number;
  focus: MapFocus | null;
}

function FreeView({ instanceId, folder, dimensionId, players, revision, focus }: FreeViewProps) {
  const source = useMemo(() => worldMapSource(instanceId, folder), [instanceId, folder]);
  const inDimension = useMemo(() => players.filter((player) => player.dimension === dimensionId), [players, dimensionId]);
  return (
    <WorldMapViewer
      source={source}
      instanceId={instanceId}
      folder={folder}
      revision={revision}
      players={inDimension}
      skinBase={worldMapSkinBase(instanceId)}
      focus={focus}
    />
  );
}
