import { Input, Label, TextField } from "@heroui/react";
import { useState, type FormEvent } from "react";
import type { Schedule } from "../../api/schedules";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { SchedulePreview } from "./SchedulePreview";
import { StepEditor } from "./StepEditor";
import { TriggerEditor } from "./TriggerEditor";

interface ScheduleFormProps {
  initial: Schedule;
  submitLabel: string;
  pending: boolean;
  error: string | null;
  onSubmit: (schedule: Schedule) => void;
  onCancel: () => void;
}

export function ScheduleForm({ initial, submitLabel, pending, error, onSubmit, onCancel }: ScheduleFormProps) {
  const [draft, setDraft] = useState(initial);
  const name = draft.name.trim();
  const canSubmit = name.length > 0 && draft.steps.length > 0 && !pending;

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (canSubmit) onSubmit({ ...draft, name });
  };

  return (
    <form onSubmit={submit} className="space-y-6">
      <TextField value={draft.name} onChange={(value: string) => setDraft({ ...draft, name: value })} className="flex flex-col gap-1.5">
        <Label className="text-sm font-medium text-ink-2">Name</Label>
        <Input placeholder="Nightly restart" maxLength={64} />
      </TextField>

      <section className="space-y-2">
        <h3 className="text-sm font-medium text-ink-2">When</h3>
        <TriggerEditor value={draft.trigger} onChange={(trigger) => setDraft({ ...draft, trigger })} />
      </section>

      <section className="space-y-2">
        <h3 className="text-sm font-medium text-ink-2">Then</h3>
        <StepEditor steps={draft.steps} onChange={(steps) => setDraft({ ...draft, steps })} />
      </section>

      <SchedulePreview schedule={name ? draft : null} />

      <div className="flex flex-wrap items-center justify-between gap-3">
        {error ? <p className="text-sm text-rose">{error}</p> : <span />}
        <div className="flex gap-2">
          <GhostButton onPress={onCancel}>Cancel</GhostButton>
          <PrimaryButton type="submit" isDisabled={!canSubmit}>
            {submitLabel}
          </PrimaryButton>
        </div>
      </div>
    </form>
  );
}
