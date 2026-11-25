import { useMemo, useState } from "react";
import {
  dimensionLabel,
  playersSummary,
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
import { mapMemory, rememberView, type MapView } from "./map-memory";
import { TileView } from "./TileView";
import { WorldMapViewer } from "./WorldMapViewer";

const NO_PLAYERS: readonly LivePlayer[] = [];

/** The built-in map: a dimension picker, the top-down tiles or the free-fly 3D view, the render status and live players. */
export function WorldMapPanel({ instanceId, canViewPlayers }: { instanceId: number; canViewPlayers: boolean }) {
  const status = useWorldMapStatus(instanceId);
  const live = useWorldMapPlayers(instanceId, canViewPlayers);
  const players = live.data?.players ?? NO_PLAYERS;
  const [picked, setPicked] = useState<string | null>(() => mapMemory(instanceId).folder ?? null);
  const [view, setViewState] = useState<MapView>(() => mapMemory(instanceId).view ?? "top");

  if (status.isPending) return <p className="text-sm text-ink-3">Loading world map…</p>;
  if (status.isError) return <p className="text-sm text-rose">{status.error.message}</p>;

  const dimensions = status.data.dimensions.filter((dimension) => dimension.regions > 0);
  const dimension = dimensions.find((entry) => entry.folder === picked) ?? dimensions[0];
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

  const tabs = dimensions.map((entry) => ({ id: entry.folder, label: dimensionLabel(entry.id) }));
  const chooseDimension = (folder: string) => {
    setPicked(folder);
    rememberView(instanceId, { folder });
  };
  const chooseView = (next: MapView) => {
    setViewState(next);
    rememberView(instanceId, { view: next });
  };

  return (
    <div className="flex min-h-[32rem] flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        {tabs.length > 1 ? (
          <SectionTabs label="Dimensions" tabs={tabs} value={dimension.folder} onChange={chooseDimension} />
        ) : (
          <p className="font-mono text-xs text-ink-3 uppercase">{dimensionLabel(dimension.id)}</p>
        )}
        <div className="flex flex-wrap items-center gap-3">
          <SectionTabs<MapView>
            label="View"
            tabs={[
              { id: "top", label: "Top" },
              { id: "free", label: "3D" },
            ]}
            value={view}
            onChange={chooseView}
          />
          <span className="font-mono text-xs text-ink-3">{summaryLine(status.data)}</span>
        </div>
      </div>
      {canViewPlayers && (
        <p className={`font-mono text-xs ${live.data?.state === "unavailable" ? "text-rose" : "text-ink-3"}`}>
          {playersSummary(live.data)}
        </p>
      )}
      <div className="relative h-[70vh] min-h-[32rem] overflow-hidden rounded-panel border border-line bg-canvas">
        {view === "top" ? (
          <TileView
            key={dimension.folder}
            instanceId={instanceId}
            folder={dimension.folder}
            dimensionId={dimension.id}
            players={players}
            revision={status.data.revision}
          />
        ) : (
          <FreeView
            key={dimension.folder}
            instanceId={instanceId}
            folder={dimension.folder}
            dimensionId={dimension.id}
            players={players}
            revision={status.data.revision}
          />
        )}
      </div>
    </div>
  );
}

interface FreeViewProps {
  instanceId: number;
  folder: string;
  dimensionId: string;
  players: readonly LivePlayer[];
  revision: number;
}

function FreeView({ instanceId, folder, dimensionId, players, revision }: FreeViewProps) {
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
    />
  );
}

function summaryLine(status: WorldMapStatus): string {
  return `${status.regions} regions · ${renderSummary(status)}`;
}
