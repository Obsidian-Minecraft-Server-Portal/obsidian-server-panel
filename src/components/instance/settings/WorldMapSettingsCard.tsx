import {
  renderSummary,
  useRenderWorldMap,
  useUpdateWorldMapSettings,
  useWorldMapSettings,
  useWorldMapStatus,
  type WorldMapSettings,
} from "../../../api/worldmap";
import { Panel } from "../../ui/Panel";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { SwitchRow } from "../../ui/SwitchRow";

export function WorldMapSettingsCard({ instanceId }: { instanceId: number }) {
  const settings = useWorldMapSettings(instanceId);
  const status = useWorldMapStatus(instanceId);
  const update = useUpdateWorldMapSettings(instanceId);
  const render = useRenderWorldMap(instanceId);

  if (settings.isPending) return <Panel title="World map"><p className="text-sm text-ink-3">Loading…</p></Panel>;
  if (settings.isError) return <Panel title="World map"><p className="text-sm text-rose">{settings.error.message}</p></Panel>;

  const current = settings.data;
  const change = (patch: Partial<WorldMapSettings>) => update.mutate({ ...current, ...patch });
  const rendering = status.data?.rendering === true;
  const error = update.error?.message ?? render.error?.message ?? status.data?.lastError;

  return (
    <Panel title="World map">
      <div className="divide-y divide-line">
        <SwitchRow
          label="Show the built-in map"
          description="Adds a World map entry to the Map tab. It renders from the world files, so it stays open while the server is offline."
          isSelected={current.enabled}
          isDisabled={update.isPending}
          onChange={(enabled) => change({ enabled })}
        />
        <SwitchRow
          label="Allow downloads from Mojang"
          description="Lets the panel download this version's client and server jars. The client jar supplies textures and block models. Nothing downloads until this is on."
          isSelected={current.consent}
          isDisabled={update.isPending}
          onChange={(consent) => change({ consent })}
        />
        <SwitchRow
          label="Update while the server runs"
          description="Re-renders regions whose files changed, every few minutes."
          isSelected={current.watchWhileRunning}
          isDisabled={update.isPending}
          onChange={(watchWhileRunning) => change({ watchWhileRunning })}
        />
        <SwitchRow
          label="Update when the server stops"
          description="Renders the regions that changed while the server was running."
          isSelected={current.renderAfterStop}
          isDisabled={update.isPending}
          onChange={(renderAfterStop) => change({ renderAfterStop })}
        />
        <SwitchRow
          label="Re-render everything once a day"
          description="A full pass rebuilds every region, which catches anything the watch missed."
          isSelected={current.fullPassHours > 0}
          isDisabled={update.isPending}
          onChange={(daily) => change({ fullPassHours: daily ? 24 : 0 })}
        />
      </div>
      <div className="mt-4 flex flex-wrap items-center justify-between gap-3 border-t border-line pt-4">
        <p className="font-mono text-xs text-ink-3">{status.data ? renderSummary(status.data) : "Checking renders…"}</p>
        <PrimaryButton isDisabled={!current.enabled || rendering || render.isPending} onPress={() => render.mutate()}>
          Render now
        </PrimaryButton>
      </div>
      {error && <p className="mt-3 text-sm text-rose">{error}</p>}
    </Panel>
  );
}
