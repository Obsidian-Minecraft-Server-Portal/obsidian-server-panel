import { useMemo, useState } from "react";
import { dimensionLabel, renderSummary, useWorldMapStatus, worldMapSource, type WorldMapStatus } from "../../api/worldmap";
import { EmptyState } from "../ui/EmptyState";
import { SectionTabs } from "../ui/SectionTabs";
import { TileView } from "./TileView";
import { WorldMapViewer } from "./WorldMapViewer";

type View = "top" | "free";

/** The built-in map: a dimension picker, the top-down tiles or the free-fly 3D view, and the render status. */
export function WorldMapPanel({ instanceId }: { instanceId: number }) {
  const status = useWorldMapStatus(instanceId);
  const [picked, setPicked] = useState<string | null>(null);
  const [view, setView] = useState<View>("top");

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
  const revision = status.data.rendering ? "busy" : `${status.data.lastFullPass ?? 0}`;

  return (
    <div className="flex min-h-[32rem] flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        {tabs.length > 1 ? (
          <SectionTabs label="Dimensions" tabs={tabs} value={dimension.folder} onChange={setPicked} />
        ) : (
          <p className="font-mono text-xs text-ink-3 uppercase">{dimensionLabel(dimension.id)}</p>
        )}
        <div className="flex flex-wrap items-center gap-3">
          <SectionTabs<View>
            label="View"
            tabs={[
              { id: "top", label: "Top" },
              { id: "free", label: "3D" },
            ]}
            value={view}
            onChange={setView}
          />
          <span className="font-mono text-xs text-ink-3">{summaryLine(status.data)}</span>
        </div>
      </div>
      <div className="relative h-[70vh] min-h-[32rem] overflow-hidden rounded-panel border border-line bg-canvas">
        {view === "top" ? (
          <TileView key={`${dimension.folder}-${revision}`} instanceId={instanceId} folder={dimension.folder} />
        ) : (
          <FreeView key={`${dimension.folder}-${revision}`} instanceId={instanceId} folder={dimension.folder} />
        )}
      </div>
    </div>
  );
}

function FreeView({ instanceId, folder }: { instanceId: number; folder: string }) {
  const source = useMemo(() => worldMapSource(instanceId, folder), [instanceId, folder]);
  return <WorldMapViewer source={source} />;
}

function summaryLine(status: WorldMapStatus): string {
  return `${status.regions} regions · ${renderSummary(status)}`;
}
