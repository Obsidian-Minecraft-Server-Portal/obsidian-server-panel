import { useMemo } from "react";
import { useDismissTask, useTasks } from "../../api/tasks";
import { TaskCard } from "./TaskCard";

interface TaskListProps {
  onNavigate: () => void;
}

export function TaskList({ onNavigate }: TaskListProps) {
  const tasks = useTasks();
  const dismiss = useDismissTask();
  const sorted = useMemo(() => [...(tasks.data ?? [])].sort((a, b) => b.id - a.id), [tasks.data]);

  if (tasks.isPending) return <div className="m-3 h-20 animate-shimmer rounded-control bg-raised" />;

  return (
    <div className="max-h-96 overflow-y-auto">
      {tasks.isError && <p className="px-4 py-8 text-center text-sm text-rose">{tasks.error.message}</p>}
      {tasks.isSuccess && sorted.length === 0 && (
        <p className="px-4 py-8 text-center text-sm text-ink-3">Long-running work appears here with live progress.</p>
      )}
      {dismiss.isError && <p className="px-4 pt-3 text-xs text-rose">{dismiss.error.message}</p>}
      <ul>
        {sorted.map((task) => (
          <li key={task.id} className="border-b border-line p-4 last:border-b-0">
            <TaskCard
              task={task}
              dismissing={dismiss.isPending && dismiss.variables === task.id}
              onDismiss={() => dismiss.mutate(task.id)}
              onNavigate={onNavigate}
            />
          </li>
        ))}
      </ul>
    </div>
  );
}
