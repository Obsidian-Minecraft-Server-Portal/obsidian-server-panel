import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { useUpdateInstance, type InstanceView } from "../../../api/instances";
import type { LoaderKind } from "../../../api/loaders";
import { LOADER_META } from "../../runtime/loaderMeta";
import { LoaderPicker } from "../../runtime/LoaderPicker";
import { LoaderVersionSelect } from "../../runtime/LoaderVersionSelect";
import { VersionSelect } from "../../runtime/VersionSelect";
import { GhostButton } from "../../ui/GhostButton";
import { Panel } from "../../ui/Panel";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { isActive } from "../status";

interface Software {
  loader: LoaderKind;
  mcVersion: string | null;
  loaderVersion: string | null;
}

function currentSoftware(view: InstanceView): Software {
  return { loader: view.loader, mcVersion: view.mc_version, loaderVersion: view.loader_version };
}

export function softwareKey(view: InstanceView): string {
  return JSON.stringify(currentSoftware(view));
}

export function SoftwarePanel({ view }: { view: InstanceView }) {
  const update = useUpdateInstance(view.id);
  const [draft, setDraft] = useState<Software>(() => currentSoftware(view));
  const [confirming, setConfirming] = useState(false);
  const busy = isActive(view.status) || view.status.installing || update.isPending;
  const changed = draft.loader !== view.loader || draft.mcVersion !== view.mc_version || draft.loaderVersion !== view.loader_version;
  const ready = draft.mcVersion !== null && draft.mcVersion.trim() !== "";
  const risks = [
    draft.mcVersion !== view.mc_version
      ? "Worlds saved by a newer Minecraft version cannot be opened by an older one. Back up first."
      : null,
    "Mods and plugins are kept, but ones that do not match the new software can stop the server from starting.",
    "The server jar and launch settings are replaced when the install finishes.",
  ].filter((risk): risk is string => risk !== null);

  const chooseLoader = (loader: LoaderKind) => {
    setDraft({ loader, mcVersion: null, loaderVersion: null });
    setConfirming(false);
  };
  const chooseVersion = (mcVersion: string | null) => setDraft((current) => ({ ...current, mcVersion, loaderVersion: null }));
  const chooseBuild = (loaderVersion: string | null) => setDraft((current) => ({ ...current, loaderVersion }));

  const apply = () => {
    if (!ready || draft.mcVersion === null) return;
    update.mutate(
      { loader: draft.loader, mc_version: draft.mcVersion.trim(), loader_version: draft.loaderVersion },
      { onSuccess: () => setConfirming(false) },
    );
  };

  return (
    <Panel title="Server software">
      <div className="space-y-5">
        <LoaderPicker value={draft.loader} onChange={chooseLoader} isDisabled={busy} exclude={["custom"]} />
        <div className="grid gap-5 md:grid-cols-2">
          <VersionSelect loader={draft.loader} value={draft.mcVersion} onChange={chooseVersion} isDisabled={busy} />
          <LoaderVersionSelect
            loader={draft.loader}
            mcVersion={draft.mcVersion}
            value={draft.loaderVersion}
            onChange={chooseBuild}
            isDisabled={busy}
          />
        </div>
        {busy && <p className="text-xs text-ink-3">Stop the server and let any install finish before changing its software.</p>}
        {update.error && <p className="text-sm text-rose">{update.error.message}</p>}
        {confirming ? (
          <div className="space-y-4 rounded-control border border-amber/40 bg-input p-4">
            <p className="flex items-center gap-2 text-sm font-medium text-ink-0">
              <Icon icon="lucide:triangle-alert" width={16} height={16} className="text-amber" />
              Reinstall as {LOADER_META[draft.loader].label} {draft.mcVersion}?
            </p>
            <ul className="list-disc space-y-1.5 pl-5 text-xs text-ink-2">
              {risks.map((risk) => (
                <li key={risk}>{risk}</li>
              ))}
            </ul>
            <div className="flex justify-end gap-2">
              <GhostButton onPress={() => setConfirming(false)}>Cancel</GhostButton>
              <PrimaryButton isPending={update.isPending} isDisabled={busy} onPress={apply}>
                Reinstall
              </PrimaryButton>
            </div>
          </div>
        ) : (
          <div className="flex justify-end gap-2">
            {changed && <GhostButton onPress={() => setDraft(currentSoftware(view))}>Discard</GhostButton>}
            <PrimaryButton isDisabled={!changed || !ready || busy} onPress={() => setConfirming(true)}>
              Review changes
            </PrimaryButton>
          </div>
        )}
      </div>
    </Panel>
  );
}
