import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { usePowerAction } from "../../api/instances";
import { useMigrations, useRollbackMigration, type RunReport, type StartedMigration } from "../../api/migration";
import { useTasks } from "../../api/tasks";
import { GhostButton } from "../ui/GhostButton";
import { Panel } from "../ui/Panel";
import { PrimaryButton } from "../ui/PrimaryButton";

interface RunStepProps {
  instanceId: number;
  started: StartedMigration;
  canStart: boolean;
  onOpenInstance: () => void;
}

function statusLine(status: string | undefined): string {
  switch (status) {
    case "finished":
      return "Migration complete";
    case "failed":
      return "Migration failed";
    case "running":
      return "Migration in progress";
    default:
      return "Starting the migration";
  }
}

export function RunStep({ instanceId, started, canStart, onOpenInstance }: RunStepProps) {
  const queryClient = useQueryClient();
  const tasks = useTasks();
  const task = tasks.data?.find((entry) => entry.id === started.task_id);
  const migrations = useMigrations(instanceId);
  const record = migrations.data?.find((entry) => entry.id === started.migration_id);
  const rollback = useRollbackMigration(instanceId);
  const power = usePowerAction(instanceId);
  const finished = task?.status === "finished";
  const failed = task?.status === "failed";
  const report = task?.result?.report as RunReport | undefined;
  const rollbackTask = rollback.data ? tasks.data?.find((entry) => entry.id === rollback.data.task_id) : undefined;
  const rollbackSettled = rollbackTask?.status === "finished" || rollbackTask?.status === "failed";

  useEffect(() => {
    if (finished || failed || rollbackSettled) void queryClient.invalidateQueries({ queryKey: ["instances"] });
  }, [finished, failed, rollbackSettled, queryClient]);

  return (
    <Panel title="Upgrade">
      <div className="space-y-5">
        <div className="space-y-2">
          <p className="text-sm text-ink-1">{statusLine(task?.status)}</p>
          <div className="h-1.5 overflow-hidden rounded-full bg-input" role="progressbar" aria-valuenow={Math.round((task?.progress ?? 0) * 100)}>
            <div className="h-full bg-accent transition-[width]" style={{ width: `${Math.round((task?.progress ?? 0) * 100)}%` }} />
          </div>
          {task?.message && <p className="font-mono text-xs text-ink-3">{task.message}</p>}
        </div>

        {finished && report && (
          <p className="text-sm text-green">
            Upgraded {report.upgraded.length} · added {report.added.length} · disabled {report.disabled.length}
          </p>
        )}
        {failed && <p className="text-sm text-rose">{task?.error ?? record?.error ?? "The migration failed."}</p>}
        {rollback.isSuccess && (
          <p className="text-sm text-ink-2">
            {rollbackSettled
              ? rollbackTask?.status === "failed"
                ? `Rollback failed: ${rollbackTask.error ?? "see the task list"}`
                : "Rollback complete."
              : `Rolling back as task ${rollback.data?.task_id ?? ""}…`}
          </p>
        )}
        {rollback.error && <p className="text-sm text-rose">{rollback.error.message}</p>}

        <div className="flex flex-wrap justify-between gap-2">
          <GhostButton onPress={onOpenInstance}>Open instance</GhostButton>
          <div className="flex flex-wrap gap-2">
            {record?.can_roll_back && (finished || failed) && !rollback.isSuccess && (
              <GhostButton isDisabled={rollback.isPending} onPress={() => rollback.mutate(record.id)}>
                Roll back
              </GhostButton>
            )}
            {finished && canStart && (
              <PrimaryButton isPending={power.isPending} onPress={() => power.mutate("start")}>
                Start server
              </PrimaryButton>
            )}
          </div>
        </div>
      </div>
    </Panel>
  );
}
