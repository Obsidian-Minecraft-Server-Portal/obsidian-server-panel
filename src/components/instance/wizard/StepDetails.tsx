import { Checkbox } from "@heroui/react";
import { FormField } from "../FormField";
import { SwitchRow } from "../../ui/SwitchRow";
import { Panel } from "../../ui/Panel";
import { isMemoryValid, isPortValid, type WizardDraft } from "./draft";

interface StepProps {
  draft: WizardDraft;
  patch: (next: Partial<WizardDraft>) => void;
}

export function StepDetails({ draft, patch }: StepProps) {
  const memoryError = !isMemoryValid(draft);
  const portError = !isPortValid(draft.port);

  return (
    <div className="grid gap-5 xl:grid-cols-[1fr_20rem]">
      <Panel title="Server details">
        <div className="grid gap-5 md:grid-cols-2">
          <FormField label="Name" value={draft.name} onChange={(name) => patch({ name })} placeholder="Survival SMP" className="md:col-span-2" />
          <FormField
            label="Port"
            type="number"
            value={draft.port}
            onChange={(port) => patch({ port })}
            placeholder="Automatic"
            hint={portError ? "Enter a port between 1 and 65535." : "Leave empty to use the next free port."}
          />
          <div className="grid grid-cols-2 gap-3">
            <FormField
              label="Memory min (MB)"
              type="number"
              value={String(draft.memoryMinMb)}
              onChange={(value) => patch({ memoryMinMb: Number(value) })}
            />
            <FormField
              label="Memory max (MB)"
              type="number"
              value={String(draft.memoryMaxMb)}
              onChange={(value) => patch({ memoryMaxMb: Number(value) })}
              hint={memoryError ? "Min must be positive and not above max." : undefined}
            />
          </div>
        </div>
      </Panel>
      <Panel title="Options">
        <div className="space-y-5">
          <SwitchRow
            label="Forward port with UPnP"
            description="Open the port on your router while the server runs."
            isSelected={draft.upnpEnabled}
            onChange={(upnpEnabled) => patch({ upnpEnabled })}
          />
          <SwitchRow
            label="Start with the panel"
            description="Start automatically after a reboot."
            isSelected={draft.autoStart}
            onChange={(autoStart) => patch({ autoStart })}
          />
          <Checkbox isSelected={draft.acceptEula} onChange={(acceptEula: boolean) => patch({ acceptEula })} className="flex items-start gap-3">
            <Checkbox.Control>
              <Checkbox.Indicator />
            </Checkbox.Control>
            <Checkbox.Content className="text-sm text-ink-2">
              I agree to the{" "}
              <a href="https://aka.ms/MinecraftEULA" target="_blank" rel="noreferrer" className="text-accent hover:underline">
                Minecraft EULA
              </a>
              .
            </Checkbox.Content>
          </Checkbox>
        </div>
      </Panel>
    </div>
  );
}
