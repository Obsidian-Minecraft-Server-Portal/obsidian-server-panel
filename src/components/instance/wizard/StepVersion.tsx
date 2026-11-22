import { FormField } from "../FormField";
import { LoaderVersionSelect } from "../../runtime/LoaderVersionSelect";
import { VersionSelect } from "../../runtime/VersionSelect";
import { Panel } from "../../ui/Panel";
import { isCustom, type WizardDraft } from "./draft";

interface StepProps {
  draft: WizardDraft;
  patch: (next: Partial<WizardDraft>) => void;
}

const FILE_INPUT =
  "w-full text-sm text-ink-2 file:mr-3 file:rounded-control file:border-0 file:bg-accent-tint file:px-3 file:py-2 file:text-accent";

export function StepVersion({ draft, patch }: StepProps) {
  if (isCustom(draft)) {
    return (
      <Panel title="Custom server jar">
        <div className="grid gap-5 md:grid-cols-2">
          <FormField
            label="Minecraft version"
            value={draft.mcVersion ?? ""}
            onChange={(mcVersion) => patch({ mcVersion: mcVersion || null })}
            placeholder="1.21.4"
            hint="Used to pick the Java runtime and filter content."
          />
          <label className="flex flex-col gap-1.5">
            <span className="text-sm font-medium text-ink-2">Server jar</span>
            <input
              type="file"
              accept=".jar,application/java-archive"
              className={FILE_INPUT}
              onChange={(event) => patch({ customJar: event.target.files?.[0] ?? null })}
            />
            <span className="text-xs text-ink-3">
              {draft.customJar ? `${draft.customJar.name} will be uploaded as server.jar.` : "Pick the server .jar to run."}
            </span>
          </label>
        </div>
      </Panel>
    );
  }

  return (
    <Panel title="Pick versions">
      <div className="grid gap-5 md:grid-cols-2">
        <VersionSelect loader={draft.loader} value={draft.mcVersion} onChange={(mcVersion) => patch({ mcVersion })} />
        <LoaderVersionSelect
          loader={draft.loader}
          mcVersion={draft.mcVersion}
          value={draft.loaderVersion}
          onChange={(loaderVersion) => patch({ loaderVersion })}
        />
      </div>
    </Panel>
  );
}
