import { Label } from "@heroui/react";
import type { Trigger } from "../../api/schedules";
import { OptionSelect } from "../runtime/OptionSelect";
import { DateTimeInput, NumberInput, TimeInput } from "./FormFields";
import { TRIGGER_OPTIONS, UNIT_OPTIONS, defaultTrigger, type TriggerKind } from "./model";
import { WeekdayPicker } from "./WeekdayPicker";

interface TriggerEditorProps {
  value: Trigger;
  onChange: (trigger: Trigger) => void;
}

export function TriggerEditor({ value, onChange }: TriggerEditorProps) {
  return (
    <div className="space-y-4">
      <OptionSelect
        label="When"
        options={TRIGGER_OPTIONS}
        value={value.kind}
        onChange={(kind) => kind && onChange(defaultTrigger(kind as TriggerKind))}
      />
      <TriggerFields value={value} onChange={onChange} />
    </div>
  );
}

function TriggerFields({ value, onChange }: TriggerEditorProps) {
  switch (value.kind) {
    case "interval":
      return (
        <div className="flex flex-wrap items-end gap-3">
          <NumberInput label="Every" min={1} value={value.every} onChange={(every) => onChange({ ...value, every })} />
          <OptionSelect
            label="Unit"
            options={UNIT_OPTIONS}
            value={value.unit}
            onChange={(unit) => unit && onChange({ ...value, unit: unit as "minutes" | "hours" })}
            className="min-w-36"
          />
        </div>
      );
    case "daily":
      return <TimeInput label="Time" value={value.time} onChange={(time) => onChange({ ...value, time })} />;
    case "weekly":
      return (
        <div className="space-y-4">
          <div className="space-y-1.5">
            <Label className="text-sm font-medium text-ink-2">Days</Label>
            <WeekdayPicker value={value.days} onChange={(days) => onChange({ ...value, days })} />
          </div>
          <TimeInput label="Time" value={value.time} onChange={(time) => onChange({ ...value, time })} />
        </div>
      );
    case "monthly":
      return (
        <div className="flex flex-wrap items-end gap-3">
          <NumberInput label="Day of month" min={1} value={value.day} max={31} onChange={(day) => onChange({ ...value, day })} />
          <TimeInput label="Time" value={value.time} onChange={(time) => onChange({ ...value, time })} />
        </div>
      );
    case "once":
      return <DateTimeInput label="Date and time" value={value.at} onChange={(at) => onChange({ ...value, at })} />;
    case "empty_for":
      return (
        <NumberInput label="Minutes empty" min={1} value={value.minutes} onChange={(minutes) => onChange({ ...value, minutes })} />
      );
    default:
      return null;
  }
}
