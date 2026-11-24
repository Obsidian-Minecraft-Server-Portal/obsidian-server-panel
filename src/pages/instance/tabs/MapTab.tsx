import { useState } from "react";
import { useInstance, useUpnp } from "../../../api/instances";
import { mapPath, useMaps, type MapKind } from "../../../api/maps";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { EmptyState } from "../../../components/ui/EmptyState";
import { SectionTabs } from "../../../components/ui/SectionTabs";

export function MapTab() {
  const id = useInstanceId();
  const instance = useInstance(id);
  const maps = useMaps(id);
  const upnp = useUpnp(id);
  const [picked, setPicked] = useState<MapKind | null>(null);
  const enabled = (maps.data ?? []).filter((map) => map.settings.enabled);
  const current = enabled.find((map) => map.kind === picked) ?? enabled[0];

  if (maps.isPending) return <p className="px-4 sm:px-8 text-sm text-ink-3">Loading map…</p>;
  if (maps.isError) return <p className="px-4 sm:px-8 text-sm text-rose">{maps.error.message}</p>;
  if (!current) {
    return (
      <div className="px-4 sm:px-8">
        <EmptyState
          icon="lucide:map"
          title="No web map is enabled"
          description="Start the server once so its map creates its config, then turn its web server on in Settings."
        />
      </div>
    );
  }

  const running = instance.data?.status.state === "running";
  const external = upnp.data?.maps.find(
    (entry) => entry.port === current.settings.port && entry.mapped && entry.external_ip && entry.external_port,
  );

  return (
    <div className="flex min-h-[32rem] flex-1 flex-col gap-3 px-4 pb-8 sm:px-8">
      <div className="flex flex-wrap items-center justify-between gap-3">
        {enabled.length > 1 ? (
          <SectionTabs
            label="Maps"
            tabs={enabled.map((map) => ({ id: map.kind, label: map.label }))}
            value={current.kind}
            onChange={setPicked}
          />
        ) : (
          <p className="font-mono text-xs text-ink-3 uppercase">{current.label}</p>
        )}
        {external && (
          <a
            href={`http://${external.external_ip}:${external.external_port}/`}
            target="_blank"
            rel="noreferrer"
            className="text-sm font-medium text-accent hover:underline"
          >
            Open full map
          </a>
        )}
      </div>
      {running ? (
        <iframe
          key={current.kind}
          src={mapPath(id, current.kind)}
          title={`${current.label} map`}
          className="min-h-[60vh] w-full flex-1 rounded-panel border border-line bg-canvas"
        />
      ) : (
        <EmptyState icon="lucide:power-off" title="The server is offline" description="Start the server to open its map." />
      )}
    </div>
  );
}
