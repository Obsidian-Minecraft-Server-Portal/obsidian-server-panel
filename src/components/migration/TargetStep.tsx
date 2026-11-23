import { useJavaRequirement } from "../../api/java";
import type { InstanceView } from "../../api/instances";
import { LOADER_META } from "../runtime/loaderMeta";
import { LoaderVersionSelect } from "../runtime/LoaderVersionSelect";
import { VersionSelect } from "../runtime/VersionSelect";
import { GhostButton } from "../ui/GhostButton";
import { Panel } from "../ui/Panel";
import { PrimaryButton } from "../ui/PrimaryButton";

const BUILD_TOOLS_LOADERS = new Set(["spigot", "craft_bukkit"]);

interface TargetStepProps {
  view: InstanceView;
  mcVersion: string | null;
  loaderVersion: string | null;
  onMcVersion: (version: string | null) => void;
  onLoaderVersion: (version: string | null) => void;
  onBack: () => void;
  onContinue: () => void;
}

export function TargetStep({
  view,
  mcVersion,
  loaderVersion,
  onMcVersion,
  onLoaderVersion,
  onBack,
  onContinue,
}: TargetStepProps) {
  const java = useJavaRequirement(mcVersion);
  const ready = mcVersion !== null && (loaderVersion !== null || view.loader === "vanilla");
  const sameVersion = mcVersion !== null && mcVersion === view.mc_version;

  return (
    <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <Panel title="Target version">
        <div className="space-y-5">
          <div className="grid gap-5 md:grid-cols-2">
            <VersionSelect loader={view.loader} value={mcVersion} onChange={onMcVersion} />
            <LoaderVersionSelect
              loader={view.loader}
              mcVersion={mcVersion}
              value={loaderVersion}
              onChange={onLoaderVersion}
            />
          </div>
          {sameVersion && (
            <p className="text-sm text-ink-3">
              This is the version the server already runs. The wizard will refresh the loader and check every mod.
            </p>
          )}
          {BUILD_TOOLS_LOADERS.has(view.loader) && (
            <p className="text-sm text-amber">BuildTools compiles this server from source, which can take several minutes.</p>
          )}
          <div className="flex flex-wrap justify-between gap-2">
            <GhostButton onPress={onBack}>Back to instance</GhostButton>
            <PrimaryButton isDisabled={!ready} onPress={onContinue}>
              Check mods
            </PrimaryButton>
          </div>
        </div>
      </Panel>

      <Panel title="Summary">
        <dl className="space-y-3 text-sm">
          <div className="flex items-center justify-between gap-4">
            <dt className="text-ink-3">Server</dt>
            <dd className="text-ink-1">{LOADER_META[view.loader].label}</dd>
          </div>
          <div className="flex items-center justify-between gap-4">
            <dt className="text-ink-3">Minecraft</dt>
            <dd className="font-mono text-xs text-ink-1">
              {view.mc_version}
              {mcVersion && mcVersion !== view.mc_version ? ` → ${mcVersion}` : ""}
            </dd>
          </div>
          <div className="flex items-center justify-between gap-4">
            <dt className="text-ink-3">Java</dt>
            <dd className="font-mono text-xs text-ink-1">
              {java.data ? `Java ${java.data.major_version} (automatic)` : mcVersion ? "Checking…" : "—"}
            </dd>
          </div>
        </dl>
        <p className="mt-5 text-xs text-ink-3">
          Worlds saved by a newer Minecraft version cannot be opened by an older one. Back up before you continue.
        </p>
      </Panel>
    </div>
  );
}
