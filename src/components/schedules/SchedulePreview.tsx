import type { Schedule } from "../../api/schedules";
import { usePreviewSchedule } from "../../api/schedules";
import { formatDateTime } from "../backups/format";
import { useDebouncedValue } from "./useDebouncedValue";

const PREVIEW_DEBOUNCE_MS = 350;

interface SchedulePreviewProps {
  schedule: Schedule | null;
}

export function SchedulePreview({ schedule }: SchedulePreviewProps) {
  const debounced = useDebouncedValue(schedule, PREVIEW_DEBOUNCE_MS);
  const preview = usePreviewSchedule(debounced);

  return (
    <div className="space-y-2 rounded-control border border-accent/30 bg-accent-tint/30 p-4">
      <p className="font-mono text-[11px] font-medium tracking-wider text-[#d9a8ff] uppercase">Preview</p>
      {!schedule ? (
        <p className="text-sm text-ink-3">Give the schedule a name to see when it will run.</p>
      ) : preview.isError ? (
        <p className="text-sm text-rose">{preview.error.message}</p>
      ) : preview.data ? (
        <>
          <p className="text-sm text-ink-0">{preview.data.description}</p>
          <ul className="flex flex-wrap gap-x-4 gap-y-1 font-mono text-[11px] text-ink-3">
            {preview.data.next_runs.map((run) => (
              <li key={run}>{formatDateTime(run)}</li>
            ))}
          </ul>
        </>
      ) : (
        <p className="text-sm text-ink-3">Calculating…</p>
      )}
    </div>
  );
}
