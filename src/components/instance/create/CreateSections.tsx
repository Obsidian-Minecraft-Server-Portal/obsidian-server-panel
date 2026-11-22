import { FormField } from "../FormField";
import { useMemoryLimits, useNextPort } from "../../../api/system";
import { MemoryRange } from "../../ui/MemoryRange";
import { JavaSelect } from "../../runtime/JavaSelect";
import { LoaderPicker } from "../../runtime/LoaderPicker";
import { LoaderVersionSelect } from "../../runtime/LoaderVersionSelect";
import { VersionSelect } from "../../runtime/VersionSelect";
import { SwitchGroup, SwitchRow } from "../../ui/SwitchRow";
import { FormSection } from "./FormSection";
import { isCustom, isMemoryRange, isPortValid, type CreateDraft } from "./draft";

export interface SectionProps {
  draft: CreateDraft;
  patch: (next: Partial<CreateDraft>) => void;
}

const FILE_INPUT =
  "w-full rounded-control border border-dashed border-line-strong bg-input/50 p-2 text-sm text-ink-2 file:mr-3 file:rounded-[9px] file:border-0 file:bg-raised file:px-3 file:py-1.5 file:text-ink-1 hover:border-line-strong";

export function ServerTypeSection({ draft, patch }: SectionProps) {
  return (
    <FormSection index={1} title={<>Server type <span className="text-rose">*</span></>} description="The software your server runs. Mod loaders and plugin servers are installed for you.">
      <LoaderPicker value={draft.loader} onChange={(loader) => patch({ loader })} />
    </FormSection>
  );
}

export function VersionSection({ draft, patch }: SectionProps) {
  return (
    <FormSection index={2} title="Version & Java" description="Type to search. Java is picked from the Minecraft version unless you choose one.">
      <div className="grid gap-5 md:grid-cols-2">
        {isCustom(draft) ? (
          <>
            <FormField
              label="Minecraft version"
              value={draft.mcVersion ?? ""}
              onChange={(mcVersion) => patch({ mcVersion: mcVersion || null })}
              placeholder="1.21.4"
              hint="Used to pick Java and filter content."
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
                {draft.customJar ? `${draft.customJar.name} is uploaded as server.jar.` : "The .jar file to run."}
              </span>
            </label>
          </>
        ) : (
          <>
            <VersionSelect loader={draft.loader} value={draft.mcVersion} onChange={(mcVersion) => patch({ mcVersion })} />
            <LoaderVersionSelect
              loader={draft.loader}
              mcVersion={draft.mcVersion}
              value={draft.loaderVersion}
              onChange={(loaderVersion) => patch({ loaderVersion })}
            />
          </>
        )}
        <div className="md:col-span-2">
          <JavaSelect
            value={draft.java}
            onChange={(java) => patch({ java })}
            allowDownload
            mcVersion={draft.mcVersion}
            description="Runtimes under “Download from Mojang” are installed with the server."
          />
        </div>
      </div>
    </FormSection>
  );
}

export function DetailsSection({ draft, patch }: SectionProps) {
  const memoryOk = isMemoryRange(draft.memoryMinMb, draft.memoryMaxMb);
  const nextPort = useNextPort(true);
  const memory = useMemoryLimits();
  const suggested = nextPort.data?.port;
  return (
    <FormSection index={3} title="Details">
      <div className="grid gap-5 md:grid-cols-3">
        <FormField
          label="Name"
          required
          value={draft.name}
          onChange={(name) => patch({ name })}
          placeholder="Survival SMP"
          className="md:col-span-3"
        />
        <FormField
          label="Port"
          type="number"
          value={draft.port}
          onChange={(port) => patch({ port })}
          placeholder={suggested ? String(suggested) : "Loading…"}
          hint={isPortValid(draft.port) ? (draft.port === "" && suggested ? `Empty uses port ${suggested}, the next free port.` : "Leave empty to use the next free port.") : "Enter a port between 1 and 65535."}
        />
        <div className="space-y-2 md:col-span-3">
          <p className="text-sm font-medium text-ink-2">
            Memory <span className="text-rose">*</span>
          </p>
          {memory.data ? (
            <MemoryRange
              limits={memory.data}
              min={draft.memoryMinMb}
              max={draft.memoryMaxMb}
              onChange={(memoryMinMb, memoryMaxMb) => patch({ memoryMinMb, memoryMaxMb })}
            />
          ) : (
            <div className="h-16 animate-pulse rounded-control bg-input/60" />
          )}
          {!memoryOk && <p className="text-xs text-rose">The minimum must be at least 1 MB and not above the maximum.</p>}
        </div>
      </div>
    </FormSection>
  );
}

export function OptionsSection({ draft, patch }: SectionProps) {
  return (
    <FormSection index={4} title="Options">
      <SwitchGroup>
        <SwitchRow
          label="Forward the port with UPnP"
          description="Opens the port on your router while the server runs."
          isSelected={draft.upnpEnabled}
          onChange={(upnpEnabled) => patch({ upnpEnabled })}
        />
        <SwitchRow
          label="Start with the panel"
          description="Starts automatically when the panel boots, for example after a reboot."
          isSelected={draft.autoStart}
          onChange={(autoStart) => patch({ autoStart })}
        />
      </SwitchGroup>
    </FormSection>
  );
}
