import { useRef, useState } from "react";
import { Icon } from "@iconify-icon/react";
import { hasPermission, useInstance, useUpnp } from "../../../api/instances";
import { useMapSession, useMaps } from "../../../api/maps";
import { useWorldMapStatus } from "../../../api/worldmap";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { WorldMapPanel } from "../../../components/worldmap/WorldMapPanel";
import { EmptyState } from "../../../components/ui/EmptyState";
import { GhostButton } from "../../../components/ui/GhostButton";
import { SectionTabs } from "../../../components/ui/SectionTabs";
import { Tip } from "../../../components/ui/Tip";
import { useFullscreen } from "../../../components/ui/useFullscreen";

const WORLD_MAP = "worldmap";

export function MapTab() {
  const id = useInstanceId();
  const instance = useInstance(id);
  const canViewWorldMap = instance.data !== undefined && hasPermission(instance.data, "map.view");
  const canViewPlayers = instance.data !== undefined && hasPermission(instance.data, "map.players");
  const worldMap = useWorldMapStatus(id, canViewWorldMap);
  const maps = useMaps(id);
  const upnp = useUpnp(id);
  const [picked, setPicked] = useState<string | null>(null);
  const frame = useRef<HTMLDivElement>(null);
  const fullscreen = useFullscreen(frame);
  const builtIn = worldMap.data?.enabled === true;
  const thirdParty = maps.isError ? [] : (maps.data ?? []).filter((map) => map.settings.enabled);
  const options: { id: string; label: string }[] = [
    ...(builtIn ? [{ id: WORLD_MAP, label: "World map" }] : []),
    ...thirdParty.map((map) => ({ id: map.kind, label: map.label })),
  ];
  const selected = options.find((option) => option.id === picked) ?? options[0];
  const current = thirdParty.find((map) => map.kind === selected?.id);
  const running = instance.data?.status.state === "running";
  const session = useMapSession(id, current?.kind, running);

  if (maps.isPending || (canViewWorldMap && worldMap.isPending)) {
    return <p className="px-4 sm:px-8 text-sm text-ink-3">Loading map…</p>;
  }
  if (maps.isError && !builtIn) return <p className="px-4 sm:px-8 text-sm text-rose">{maps.error.message}</p>;
  if (!selected) {
    return (
      <div className="px-4 sm:px-8">
        <EmptyState
          icon="lucide:map"
          title="No map is enabled"
          description="Turn on the World map in Settings, or start the server once so a web map can create its config."
        />
      </div>
    );
  }

  const tabs =
    options.length > 1 ? (
      <SectionTabs label="Maps" tabs={options} value={selected.id} onChange={setPicked} />
    ) : null;

  if (selected.id === WORLD_MAP) {
    return (
      <div className="flex min-h-[32rem] flex-1 flex-col gap-3 px-4 pb-8 sm:px-8">
        {tabs}
        <WorldMapPanel instanceId={id} canViewPlayers={canViewPlayers} />
      </div>
    );
  }

  if (!current) {
    return (
      <div className="px-4 sm:px-8">
        <EmptyState icon="lucide:map" title="That map is not enabled" description="Pick another map above." />
      </div>
    );
  }

  const external = upnp.data?.maps.find(
    (entry) => entry.port === current.settings.port && entry.mapped && entry.external_ip && entry.external_port,
  );

  return (
    <div className="flex min-h-[32rem] flex-1 flex-col gap-3 px-4 pb-8 sm:px-8">
      <div className="flex flex-wrap items-center justify-between gap-3">
        {tabs ?? <p className="font-mono text-xs text-ink-3 uppercase">{current.label}</p>}
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
