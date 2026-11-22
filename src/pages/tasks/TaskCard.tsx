import { formatDistanceToNow } from "date-fns";
import type { TaskStatus, TaskView } from "../../api/tasks";
import { Chip } from "../../components/ui/Chip";
import { GhostButton } from "../../components/ui/GhostButton";
import { ProgressBar } from "../../components/ui/ProgressBar";
import type { Tone } from "../../components/ui/tone";

const STATUS: Record<TaskStatus, { label: string; tone: Tone }> = {
  running: { label: "Running", tone: "accent" },
  finished: { label: "Finished", tone: "cyan" },
  failed: { label: "Failed", tone: "rose" },
};

interface TaskCardProps {
  task: TaskView;
  dismissing: boolean;
  onDismiss: () => void;
}

export function TaskCard({ task, dismissing, onDismiss }: TaskCardProps) {
  const status = STATUS[task.status];
  const running = task.status === "running";

  return (
    <article className="animate-tick-in space-y-4 rounded-panel border border-line bg-panel p-5">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 space-y-2">
          <h3 className="truncate text-base font-semibold text-ink-0">{task.title}</h3>
          <div className="flex flex-wrap gap-2">
            <Chip tone={status.tone} pulse={running}>
              {status.label}
            </Chip>
            <Chip>{task.kind}</Chip>
            {task.owner === null && <Chip tone="ember">Global</Chip>}
          </div>
        </div>
        {!running && (
          <GhostButton size="sm" isDisabled={dismissing} onPress={onDismiss}>
            Dismiss
          </GhostButton>
        )}
      </header>

      {running && (
        <div className="flex items-center gap-3">
          <ProgressBar value={task.progress * 100} className="flex-1" />
          <span className="font-mono text-xs text-ink-3">{Math.round(task.progress * 100)}%</span>
        </div>
      )}
      {task.message && <p className="text-sm text-ink-2">{task.message}</p>}
      {task.error && (
        <pre className="rounded-control border border-rose/30 bg-rose/10 p-3 font-mono text-xs whitespace-pre-wrap text-rose">
          {task.error}
        </pre>
      )}
      {task.result && (
        <pre className="max-h-60 overflow-auto rounded-control border border-line bg-input p-3 font-mono text-xs text-ink-2">
          {JSON.stringify(task.result, null, 2)}
        </pre>
      )}
      <p className="font-mono text-[11px] text-ink-4 uppercase">
        #{task.id} · updated {formatDistanceToNow(new Date(task.updated_at), { addSuffix: true })}
      </p>
    </article>
  );
}
