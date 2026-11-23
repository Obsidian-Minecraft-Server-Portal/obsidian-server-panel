import { useScheduleRuns, type RunStatus } from "../../api/schedules";
import { formatDateTime } from "../backups/format";
import { Chip } from "../ui/Chip";
import type { Tone } from "../ui/tone";

const RUN_TONE: Record<RunStatus, Tone> = {
  running: "amber",
  succeeded: "green",
  failed: "rose",
};

interface RunHistoryProps {
  scheduleId: number;
}

export function RunHistory({ scheduleId }: RunHistoryProps) {
  const runs = useScheduleRuns(scheduleId);

  if (runs.isError) return <p className="text-sm text-rose">{runs.error.message}</p>;
  if (!runs.data) return <p className="font-mono text-xs text-ink-3">LOADING RUNS…</p>;
  if (runs.data.length === 0) return <p className="text-sm text-ink-3">No runs yet.</p>;

  return (
    <ul className="divide-y divide-line rounded-control border border-line">
      {runs.data.map((run) => (
        <li key={run.id} className="flex flex-wrap items-center gap-3 px-3 py-2">
          <Chip tone={RUN_TONE[run.status]} pulse={run.status === "running"}>
            {run.status}
          </Chip>
          <span className="font-mono text-[11px] text-ink-3">{formatDateTime(run.started_at)}</span>
          <span className="min-w-0 flex-1 truncate text-xs text-ink-2">{run.message}</span>
        </li>
      ))}
    </ul>
  );
}
