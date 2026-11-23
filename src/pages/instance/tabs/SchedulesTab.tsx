import { useState } from "react";
import { useParams } from "react-router";
import {
  useCreateSchedule,
  useDeleteSchedule,
  useRunSchedule,
  useSchedules,
  useUpdateSchedule,
  type Schedule,
} from "../../../api/schedules";
import { PresetCards } from "../../../components/schedules/PresetCards";
import { ScheduleCard } from "../../../components/schedules/ScheduleCard";
import { ScheduleForm } from "../../../components/schedules/ScheduleForm";
import { blankSchedule } from "../../../components/schedules/model";
import type { Preset } from "../../../components/schedules/presets";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Panel } from "../../../components/ui/Panel";
import { PrimaryButton } from "../../../components/ui/PrimaryButton";

interface Editor {
  recordId: number | null;
  initial: Schedule;
}

export function SchedulesTab() {
  const { id = "" } = useParams();
  const instanceId = Number(id);
  const schedules = useSchedules(instanceId);
  const create = useCreateSchedule(instanceId);
  const update = useUpdateSchedule(instanceId);
  const remove = useDeleteSchedule(instanceId);
  const run = useRunSchedule();

  const [editor, setEditor] = useState<Editor | null>(null);
  const [formKey, setFormKey] = useState(0);

  const busy = create.isPending || update.isPending || remove.isPending || run.isPending;
  const failure = remove.error ?? run.error ?? (editor ? null : update.error);
  const formError = (create.error ?? (editor ? update.error : null))?.message ?? null;

  const openEditor = (next: Editor) => {
    create.reset();
    update.reset();
    setEditor(next);
    setFormKey((key) => key + 1);
  };
  const closeEditor = () => setEditor(null);

  const submit = (schedule: Schedule) => {
    if (!editor) return;
    if (editor.recordId === null) {
      create.mutate(schedule, { onSuccess: closeEditor });
    } else {
      update.mutate({ id: editor.recordId, patch: schedule }, { onSuccess: closeEditor });
    }
  };

  const list = schedules.data ?? [];

  return (
    <div className="space-y-6 px-8 pb-8">
      <Panel
        title="Schedules"
        actions={
          editor ? null : (
            <PrimaryButton onPress={() => openEditor({ recordId: null, initial: blankSchedule() })}>New schedule</PrimaryButton>
          )
        }
      >
        {failure && <p className="mb-4 text-sm text-rose">{failure.message}</p>}

        {editor && (
          <div className="mb-6 space-y-6 rounded-panel border border-line-strong bg-raised/30 p-5">
            <h3 className="text-base font-semibold text-ink-0">
              {editor.recordId === null ? "New schedule" : "Edit schedule"}
            </h3>
            {editor.recordId === null && (
              <div className="space-y-3">
                <p className="font-mono text-xs text-ink-3">START FROM A PRESET</p>
                <PresetCards onPick={(preset: Preset) => openEditor({ recordId: null, initial: preset.schedule })} />
              </div>
            )}
            <ScheduleForm
              key={formKey}
              initial={editor.initial}
              submitLabel={editor.recordId === null ? "Create schedule" : "Save changes"}
              pending={create.isPending || update.isPending}
              error={formError}
              onSubmit={submit}
              onCancel={closeEditor}
            />
          </div>
        )}

        {!schedules.data && !schedules.isError && <p className="font-mono text-xs text-ink-3">LOADING SCHEDULES…</p>}
        {schedules.isError && <p className="text-sm text-rose">{schedules.error.message}</p>}

        {schedules.data && list.length === 0 && !editor && (
          <EmptyState
            icon="lucide:calendar-clock"
            title="No schedules yet"
            description="Schedules run backups, restarts and commands on a timer or in response to server events. Pick a preset to start."
            action={
              <div className="w-full max-w-5xl">
                <PresetCards onPick={(preset) => openEditor({ recordId: null, initial: preset.schedule })} />
              </div>
            }
          />
        )}

        {list.length > 0 && (
          <ul className="space-y-3">
            {list.map((record) => (
              <li key={record.id}>
                <ScheduleCard
                  record={record}
                  busy={busy}
                  onToggle={(enabled) => update.mutate({ id: record.id, patch: { enabled } })}
                  onRun={() => run.mutate(record.id)}
                  onEdit={() => openEditor({ recordId: record.id, initial: record.schedule })}
                  onDelete={() => remove.mutate(record.id)}
                />
              </li>
            ))}
          </ul>
        )}
      </Panel>
    </div>
  );
}
