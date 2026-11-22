import { useMemo } from "react";
import { useDismissTask, useTasks } from "../../api/tasks";
import { EmptyState } from "../../components/ui/EmptyState";
import { PageHeader } from "../../components/ui/PageHeader";
import { TaskCard } from "./TaskCard";

export function TasksPage() {
  const tasks = useTasks();
  const dismiss = useDismissTask();
  const sorted = useMemo(
    () => [...(tasks.data ?? [])].sort((a, b) => b.id - a.id),
    [tasks.data],
  );

  return (
    <>
      <PageHeader title="Tasks" subtitle="Downloads, installs, builds, backups and restores in progress." />
      <div className="max-w-4xl space-y-3 px-8 pb-8">
        {tasks.isError && (
          <EmptyState icon="lucide:triangle-alert" title="Tasks unavailable" description={tasks.error.message} />
        )}
        {tasks.isSuccess && sorted.length === 0 && (
          <EmptyState icon="lucide:list-checks" title="No tasks" description="Long-running work appears here with live progress." />
        )}
        {dismiss.isError && <p className="text-sm text-rose">{dismiss.error.message}</p>}
        {sorted.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            dismissing={dismiss.isPending && dismiss.variables === task.id}
            onDismiss={() => dismiss.mutate(task.id)}
          />
        ))}
      </div>
    </>
  );
}
