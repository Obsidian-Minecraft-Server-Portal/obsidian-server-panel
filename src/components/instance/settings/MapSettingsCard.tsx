import { useState } from "react";
import { Link } from "react-router";
import { useAcceptBlueMapDownload, useUpdateMap, type DetectedMap } from "../../../api/maps";
import { FormField } from "../FormField";
import { parentPath } from "../../files/fileUtils";
import { Panel } from "../../ui/Panel";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { SwitchRow } from "../../ui/SwitchRow";
import { MapAdvancedSettings } from "./MapAdvancedSettings";

export function MapSettingsCard({ instanceId, map }: { instanceId: number; map: DetectedMap }) {
  const update = useUpdateMap(instanceId);
  const accept = useAcceptBlueMapDownload(instanceId);
  const [enabled, setEnabled] = useState(map.settings.enabled);
  const [port, setPort] = useState(String(map.settings.port));
  const [bind, setBind] = useState(map.settings.bind ?? "");
  const portNumber = Number(port);
  const portValid = Number.isInteger(portNumber) && portNumber >= 1 && portNumber <= 65535;
  const dirty = enabled !== map.settings.enabled || port !== String(map.settings.port) || bind !== (map.settings.bind ?? "");

  const save = () => {
    update.mutate({
      kind: map.kind,
      update: { enabled, port: portNumber, bind: bind.trim() || undefined },
    });
  };

  return (
    <Panel title={`${map.label} web server`}>
      <div className="space-y-5">
        <p className="font-mono text-xs text-ink-3">{map.config_path ?? "Start the server once so the map creates its config."}</p>
        {map.error && <p className="text-sm text-rose">{map.error}</p>}
        {map.editable ? (
          <>
            <SwitchRow
              label="Serve the map"
              description="Turn this off to stop the map web server."
              isSelected={enabled}
              onChange={setEnabled}
              isDisabled={update.isPending}
            />
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <FormField
                label="Port"
                type="number"
                value={port}
                onChange={setPort}
                hint="Must differ from the server port."
                isDisabled={update.isPending}
              />
              <FormField
                label="Bind address"
                value={bind}
                onChange={setBind}
                placeholder="0.0.0.0"
                hint="Leave empty to keep the value in the config."
                isDisabled={update.isPending}
              />
            </div>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="text-xs text-ink-3">Changes apply the next time the server starts.</p>
              <PrimaryButton isDisabled={!dirty || !portValid} isPending={update.isPending} onPress={save}>
                Save map settings
              </PrimaryButton>
            </div>
            <MapAdvancedSettings instanceId={instanceId} kind={map.kind} />
          </>
        ) : (
          <div className="space-y-3">
            <p className="text-sm text-ink-2">
              {map.label} keeps its settings in a HOCON file that Obsidian does not edit yet. Its web server is{" "}
              {map.settings.enabled ? "on" : "off"} (port {map.settings.port}).
            </p>
            {map.download_accepted === false && (
              <div className="space-y-3 rounded-control border border-amber/40 bg-input px-3 py-3">
                <p className="text-sm text-ink-2">
                  BlueMap needs Minecraft's client file to render the map. Accepting lets it download that file from Mojang's servers. By accepting, you confirm that you own a license to Minecraft (Java Edition) and agree to Mojang's EULA.
                </p>
                <div className="flex flex-wrap items-center justify-between gap-3">
                  <p className="text-xs text-ink-3">Takes effect the next time the server starts.</p>
                  <PrimaryButton isPending={accept.isPending} onPress={() => accept.mutate()}>
                    Accept Mojang's download
                  </PrimaryButton>
                </div>
                {accept.error && <p className="text-sm text-rose">{accept.error.message}</p>}
              </div>
            )}
            {map.config_path && (
              <Link
                to={`/instances/${instanceId}/content?${new URLSearchParams({ section: "files", path: parentPath(map.config_path) })}`}
                className="inline-flex text-sm font-medium text-accent hover:underline"
              >
                Open its folder in Files
              </Link>
            )}
            {accept.isSuccess && (
              <p className="text-sm text-ink-2">Accepted. BlueMap can download its client file after the next server start.</p>
            )}
          </div>
        )}
        {update.error && <p className="text-sm text-rose">{update.error.message}</p>}
      </div>
    </Panel>
  );
}
