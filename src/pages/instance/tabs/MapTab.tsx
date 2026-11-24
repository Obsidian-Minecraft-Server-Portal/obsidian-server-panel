import { useRef, useState } from "react";
import { Icon } from "@iconify-icon/react";
import { useInstance, useUpnp } from "../../../api/instances";
import { useMapSession, useMaps, type MapKind } from "../../../api/maps";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { EmptyState } from "../../../components/ui/EmptyState";
import { GhostButton } from "../../../components/ui/GhostButton";
import { SectionTabs } from "../../../components/ui/SectionTabs";
import { Tip } from "../../../components/ui/Tip";
import { useFullscreen } from "../../../components/ui/useFullscreen";

export function MapTab() {
  const id = useInstanceId();
  const instance = useInstance(id);
  const maps = useMaps(id);
  const upnp = useUpnp(id);
  const [picked, setPicked] = useState<MapKind | null>(null);
  const frame = useRef<HTMLDivElement>(null);
  const fullscreen = useFullscreen(frame);
  const enabled = (maps.data ?? []).filter((map) => map.settings.enabled);
  const current = enabled.find((map) => map.kind === picked) ?? enabled[0];
  const running = instance.data?.status.state === "running";
  const session = useMapSession(id, current?.kind, running);

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
        <div className="flex items-center gap-3">
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
          {running && fullscreen.supported && (
            <Tip label="Full screen" placement="bottom">
              <GhostButton isIconOnly size="sm" aria-label="Full screen" onPress={fullscreen.toggle}>
                <Icon icon="lucide:maximize" width={16} height={16} />
              </GhostButton>
            </Tip>
          )}
        </div>
      </div>
      {running ? (
        <div
          ref={frame}
          className="flex min-h-[60vh] flex-1 flex-col overflow-hidden rounded-panel border border-line bg-canvas [&:fullscreen]:rounded-none [&:fullscreen]:border-0"
        >
          {fullscreen.active && (
            <div className="flex items-center justify-between gap-3 border-b border-line bg-rail px-3 py-2">
              <p className="font-mono text-xs text-ink-3 uppercase">{current.label}</p>
              <GhostButton isIconOnly size="sm" aria-label="Exit full screen" onPress={fullscreen.toggle}>
                <Icon icon="lucide:minimize" width={16} height={16} />
              </GhostButton>
            </div>
          )}
          {session.data ? (
            <iframe
              key={current.kind}
              src={session.data.path}
              title={`${current.label} map`}
              sandbox="allow-scripts allow-popups allow-forms allow-pointer-lock"
              allowFullScreen
              className="min-h-0 w-full flex-1"
            />
          ) : (
            <p className={`p-4 text-sm ${session.isError ? "text-rose" : "text-ink-3"}`}>
              {session.isError ? session.error.message : "Opening map…"}
            </p>
          )}
        </div>
      ) : (
        <EmptyState icon="lucide:power-off" title="The server is offline" description="Start the server to open its map." />
      )}
    </div>
  );
}
