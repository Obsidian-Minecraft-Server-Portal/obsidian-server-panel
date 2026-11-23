import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { useProperties, useSaveProperties } from "../../../api/instances";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { PropertyFields } from "../../../components/instance/properties/PropertyFields";
import { RawPropertiesEditor } from "../../../components/instance/properties/RawPropertiesEditor";
import {
  parseProperties,
  sameProperties,
  serializeProperties,
  type Properties,
} from "../../../components/instance/properties/propertiesText";
import { GhostButton } from "../../../components/ui/GhostButton";
import { PrimaryButton } from "../../../components/ui/PrimaryButton";

export function PropertiesTab() {
  const id = useInstanceId();
  const saved = useProperties(id);
  const save = useSaveProperties(id);
  const [edits, setEdits] = useState<Properties | null>(null);
  const [raw, setRaw] = useState<string | null>(null);

  const base = saved.data ?? {};
  const current = raw !== null ? parseProperties(raw) : (edits ?? base);
  const dirty = saved.data !== undefined && !sameProperties(current, base);
  const rawMode = raw !== null;

  const setRawMode = (on: boolean) => {
    if (on) {
      setRaw(serializeProperties(current));
    } else {
      setEdits(parseProperties(raw ?? ""));
      setRaw(null);
    }
  };

  const update = (key: string, value: string) => setEdits({ ...current, [key]: value });

  const discard = () => {
    setEdits(null);
    setRaw(null);
  };

  if (saved.isError) return <p className="px-4 sm:px-8 text-sm text-rose">{saved.error.message}</p>;
  if (!saved.data) return <p className="px-4 sm:px-8 text-sm text-ink-3">Loading server.properties…</p>;

  return (
    <div className="space-y-5 px-4 sm:px-8 pb-8">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div role="group" aria-label="Editor mode" className="flex rounded-control border border-line bg-input p-1">
          {(["Form", "Raw"] as const).map((label) => (
            <Button
              key={label}
              type="button"
              variant="ghost"
              aria-pressed={rawMode === (label === "Raw")}
              onPress={() => setRawMode(label === "Raw")}
              className={cn(
                "flex items-center gap-2 rounded-[9px] px-3 py-1.5 text-sm transition-colors",
                rawMode === (label === "Raw")
                  ? "bg-raised font-medium text-ink-0 shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]"
                  : "bg-transparent text-ink-3 hover:bg-raised hover:text-ink-1",
              )}
            >
              <Icon icon={label === "Form" ? "lucide:sliders-horizontal" : "lucide:code"} width={14} height={14} />
              {label}
            </Button>
          ))}
        </div>
        <div className="flex items-center gap-3">
          {save.error && <span className="text-sm text-rose">{save.error.message}</span>}
          <span className="font-mono text-xs text-ink-3">{dirty ? "UNSAVED CHANGES" : "SAVED"}</span>
          <GhostButton isDisabled={!dirty || save.isPending} onPress={discard}>
            Discard
          </GhostButton>
          <PrimaryButton isDisabled={!dirty} isPending={save.isPending} onPress={() => save.mutate(current, { onSuccess: () => setEdits(null) })}>
            Save changes
          </PrimaryButton>
        </div>
      </div>
      <p className="text-xs text-ink-3">Changes apply the next time the server starts.</p>
      {rawMode ? (
        <RawPropertiesEditor text={raw ?? ""} onChange={setRaw} disabled={save.isPending} />
      ) : (
        <PropertyFields values={current} onChange={update} disabled={save.isPending} />
      )}
    </div>
  );
}
