import { cn } from "@heroui/react";
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
import { useFullscreen } from "../ui/useFullscreen";
import type { FollowMode, MapFocus } from "./map-focus";
import { mapMemory, rememberView, type MapView } from "./map-memory";
import { MapStats } from "./map-stats";
import { MapStatsCard } from "./MapStatsCard";
import { MapToolbarButton } from "./MapToolbarButton";
import { type Following, PlayersPanel } from "./PlayersPanel";
import { TileView } from "./TileView";
import { useViewerHeartbeat } from "./use-viewer-heartbeat";
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
  const [following, setFollowing] = useState<Following | null>(null);
  const [playersOpen, setPlayersOpen] = useState(false);
  const [statsOpen, setStatsOpen] = useState(false);
  const nonce = useRef(0);
  const frame = useRef<HTMLDivElement>(null);
  const stats = useMemo(() => new MapStats(), []);
  const fullscreen = useFullscreen(frame);
  const attachFrame = useCallback((node: HTMLDivElement | null) => {
    frame.current = node;
    return keepWheelInside(node);
  }, []);

  const dimensions = status.data?.dimensions.filter((dimension) => dimension.regions > 0) ?? [];
  const dimension = dimensions.find((entry) => entry.folder === picked) ?? dimensions[0];
  const reportView = useViewerHeartbeat(instanceId, {
    folder: dimension?.folder ?? "",
    mode: view,
    follow: following?.name ?? null,
  });
  const focus = useMemo<MapFocus | null>(
    () =>
      request
        ? {
            name: request.name,
            nonce: request.nonce,
            follow: following && following.name === request.name ? following.mode : null,
          }
        : null,
    [request, following],
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
  const followPlayer = (player: LivePlayer, mode: FollowMode) => {
    if (following?.name === player.name && following.mode === mode) {
      setFollowing(null);
      return;
    }
    setFollowing({ name: player.name, mode });
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
      <div
        ref={attachFrame}
        data-native-scroll
        className="relative h-[65vh] min-h-[24rem] flex-1 overflow-hidden rounded-panel border border-line bg-canvas lg:h-[72vh] lg:min-h-[32rem]"
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
            onPlace={reportView}
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
            stats={stats}
            onPlace={reportView}
            onInput={() => setFollowing(null)}
          />
        )}
        <div className="absolute top-3 left-3 flex items-start gap-2">
          <MapToolbarButton label="Top-down" icon="lucide:map" pressed={view === "top"} onPress={() => chooseView("top")} />
          <MapToolbarButton label="3D" icon="lucide:box" pressed={view === "free"} onPress={() => chooseView("free")} />
          <MapToolbarButton
            label="Map statistics"
            icon="lucide:info"
            pressed={statsOpen}
            onPress={() => setStatsOpen((open) => !open)}
          />
          {fullscreen.supported && (
            <MapToolbarButton
              label="Fullscreen"
              icon={fullscreen.active ? "lucide:minimize" : "lucide:maximize"}
              pressed={fullscreen.active}
              onPress={fullscreen.toggle}
            />
          )}
          {canViewPlayers && (
            <MapToolbarButton
              label="Players"
              icon="lucide:users"
              pressed={playersOpen}
              badge={players.length}
              onPress={() => setPlayersOpen((open) => !open)}
            />
          )}
        </div>
        {(statsOpen || (canViewPlayers && playersOpen)) && (
          <div
            className={cn(
              "pointer-events-none absolute top-14 left-3 z-10 flex w-80 max-w-[calc(100%-1.5rem)] flex-col gap-2",
              view === "free" ? "bottom-16" : "bottom-3",
            )}
          >
            {statsOpen && (
              <div className="pointer-events-auto self-start">
                <MapStatsCard instanceId={instanceId} stats={stats} live={view === "free"} />
              </div>
            )}
            {canViewPlayers && playersOpen && (
              <PlayersPanel
                live={live.data}
                dimensionId={dimension.id}
                skinBase={worldMapSkinBase(instanceId)}
                focused={request?.name ?? null}
                following={following}
                onClose={() => setPlayersOpen(false)}
                onFocus={focusPlayer}
                onFollow={followPlayer}
              />
            )}
          </div>
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
  stats: MapStats;
  onPlace: (place: { x: number; y: number; z: number }) => void;
  onInput: () => void;
}

function FreeView({ instanceId, folder, dimensionId, players, revision, focus, stats, onPlace, onInput }: FreeViewProps) {
  const source = useMemo(() => worldMapSource(instanceId, folder), [instanceId, folder]);
  return (
    <WorldMapViewer
      source={source}
      instanceId={instanceId}
      folder={folder}
      dimensionId={dimensionId}
      revision={revision}
      players={players}
      skinBase={worldMapSkinBase(instanceId)}
      focus={focus}
      stats={stats}
      onPlace={onPlace}
      onInput={onInput}
    />
  );
}
