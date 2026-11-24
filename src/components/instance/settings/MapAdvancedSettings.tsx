import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { type ConfigEntry, type MapKind, useMapEntries, useUpdateMapEntry } from "../../../api/maps";
import { FormField } from "../FormField";
import { GhostButton } from "../../ui/GhostButton";
import { SwitchRow } from "../../ui/SwitchRow";

export function MapAdvancedSettings({ instanceId, kind, editable }: { instanceId: number; kind: MapKind; editable: boolean }) {
  const [open, setOpen] = useState(false);
  const entries = useMapEntries(instanceId, kind, open);
  const update = useUpdateMapEntry(instanceId, kind);

  return (
    <div className="space-y-4 border-t border-line pt-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="text-sm font-medium text-ink-1">All settings</p>
          <p className="text-xs text-ink-3">{editable ? "Every value in the config file. A wrong value can stop the map from loading." : "Every value in the config file. Edit the file in Files to change one."}</p>
        </div>
        <GhostButton size="sm" onPress={() => setOpen((value) => !value)}>
          <Icon icon={open ? "lucide:chevron-up" : "lucide:chevron-down"} width={15} height={15} />
          {open ? "Hide" : "Show"}
        </GhostButton>
      </div>
      {open && entries.isPending && <p className="text-sm text-ink-3">Reading the config…</p>}
      {open && entries.isError && <p className="text-sm text-rose">{entries.error.message}</p>}
      {open && entries.data && (
        <div className="max-h-[28rem] space-y-4 overflow-y-auto pr-4">
          {entries.data.map((entry) => (
            <EntryField
              key={`${entry.path}-${entry.value}`}
              entry={entry}
              disabled={!editable || update.isPending}
              onSave={(value) => update.mutate({ path: entry.path, value })}
            />
          ))}
        </div>
      )}
      {update.error && <p className="text-sm text-rose">{update.error.message}</p>}
    </div>
  );
}

function EntryField({ entry, disabled, onSave }: { entry: ConfigEntry; disabled: boolean; onSave: (value: string) => void }) {
  const [draft, setDraft] = useState(entry.value);
  if (entry.kind === "bool") {
    return (
      <SwitchRow
        label={entry.path}
        isSelected={entry.value === "true"}
        isDisabled={disabled}
        onChange={(selected) => onSave(String(selected))}
      />
    );
  }
  const dirty = draft !== entry.value;
  return (
    <div className="flex items-end gap-3">
      <div className="min-w-0 flex-1">
        <FormField
          label={entry.path}
          type={entry.kind === "number" ? "number" : "text"}
          value={draft}
          onChange={setDraft}
          isDisabled={disabled}
        />
      </div>
      {dirty && (
        <GhostButton size="sm" isDisabled={disabled} onPress={() => onSave(draft)}>
          Save
        </GhostButton>
      )}
    </div>
  );
}
