import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { formatDistanceToNow } from "date-fns";
import { Link } from "react-router";
import type { TaskStatus, TaskView } from "../../api/tasks";
import { Chip } from "../../components/ui/Chip";
import { GhostButton } from "../../components/ui/GhostButton";
import { ProgressBar } from "../../components/ui/ProgressBar";
import type { Tone } from "../../components/ui/tone";

const STATUS: Record<TaskStatus, { label: string; tone: Tone; icon: string; badge: string }> = {
  running: { label: "Running", tone: "accent", icon: "lucide:loader-circle", badge: "bg-accent-tint text-accent" },
  finished: { label: "Done", tone: "cyan", icon: "lucide:check", badge: "bg-cyan/12 text-cyan" },
  failed: { label: "Failed", tone: "rose", icon: "lucide:x", badge: "bg-rose/12 text-rose" },
};

function instanceLink(task: TaskView): string | null {
  const id = task.result?.instance_id;
  return typeof id === "number" ? `/instances/${id}` : null;
}

interface TaskCardProps {
  task: TaskView;
  dismissing: boolean;
  onDismiss: () => void;
}

export function TaskCard({ task, dismissing, onDismiss }: TaskCardProps) {
  const status = STATUS[task.status];
  const running = task.status === "running";
  const link = instanceLink(task);
  const detail = running ? task.message : task.status === "finished" ? "Completed" : null;

  return (
    <article className="flex animate-tick-in gap-4 rounded-panel border border-line bg-gradient-to-b from-panel to-rail p-5 glass-hairline">
      <span className={cn("grid size-10 shrink-0 place-items-center rounded-control", status.badge)}>
        <Icon icon={status.icon} width={18} height={18} className={running ? "animate-spin" : undefined} />
      </span>
      <div className="min-w-0 flex-1 space-y-3">
        <header className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0 space-y-1.5">
            <h3 className="truncate text-base font-semibold text-ink-0">{task.title}</h3>
            <div className="flex flex-wrap items-center gap-2">
              <Chip tone={status.tone} pulse={running}>
                {status.label}
              </Chip>
              <Chip>{task.kind.replace(/[._]/g, " ")}</Chip>
              {task.owner === null && <Chip tone="ember">Global</Chip>}
            </div>
          </div>
          <div className="flex items-center gap-2">
            {link && !running && (
              <Link to={link} className="rounded-control px-3 py-1.5 text-sm font-medium text-accent hover:bg-accent-tint">
                Open
              </Link>
            )}
            {!running && (
              <GhostButton size="sm" isDisabled={dismissing} onPress={onDismiss}>
                Dismiss
              </GhostButton>
            )}
          </div>
        </header>

        {running && (
          <div className="flex items-center gap-3">
            <ProgressBar value={task.progress * 100} className="flex-1" />
            <span className="w-10 text-right font-mono text-xs text-ink-3">{Math.round(task.progress * 100)}%</span>
          </div>
        )}
        {detail && <p className="text-sm text-ink-2">{detail}</p>}
        {task.error && (
          <pre className="rounded-control border border-rose/30 bg-rose/10 p-3 font-mono text-xs whitespace-pre-wrap text-rose">
            {task.error}
          </pre>
        )}
        <p className="font-mono text-[11px] text-ink-4 uppercase">
          #{task.id} · updated {formatDistanceToNow(new Date(task.updated_at), { addSuffix: true })}
        </p>
      </div>
    </article>
  );
}
