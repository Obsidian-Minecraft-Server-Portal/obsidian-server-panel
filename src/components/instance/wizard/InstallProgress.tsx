import { useEffect } from "react";
import { useNavigate } from "react-router";
import { useReinstallInstance, type InstanceView } from "../../../api/instances";
import { useTasks, type TaskView } from "../../../api/tasks";
import { GhostButton } from "../../ui/GhostButton";
import { Panel } from "../../ui/Panel";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { ProgressBar } from "../../ui/ProgressBar";

const INSTALL_TASK = "instance.install";

function latestInstallTask(tasks: TaskView[] | undefined, name: string): TaskView | undefined {
  return tasks
    ?.filter((task) => task.kind === INSTALL_TASK && task.title === `Installing ${name}`)
    .sort((a, b) => b.id - a.id)[0];
}

export function InstallProgress({ view }: { view: InstanceView }) {
  const navigate = useNavigate();
  const tasks = useTasks();
  const reinstall = useReinstallInstance(view.id);
  const task = latestInstallTask(tasks.data, view.name);

  useEffect(() => {
    if (task?.status === "finished") navigate(`/instances/${view.id}`);
  }, [task?.status, view.id, navigate]);

  const failed = task?.status === "failed";
  const progress = Math.round((task?.progress ?? 0) * 100);

  return (
    <Panel title={`Installing ${view.name}`}>
      <div className="space-y-4">
        <div className="flex items-center justify-between gap-3 font-mono text-xs text-ink-3">
          <span className="truncate">{task?.message || "Preparing installation…"}</span>
          <span>{failed ? "FAILED" : `${progress}%`}</span>
        </div>
        <ProgressBar value={progress} />
        {failed && <p className="text-sm text-rose">{task?.error ?? "The installation failed."}</p>}
        {reinstall.error && <p className="text-sm text-rose">{reinstall.error.message}</p>}
        <div className="flex gap-2">
          {failed && (
            <PrimaryButton isPending={reinstall.isPending} onPress={() => reinstall.mutate()}>
              Retry install
            </PrimaryButton>
          )}
          <GhostButton onPress={() => navigate(`/instances/${view.id}`)}>Open instance</GhostButton>
        </div>
      </div>
    </Panel>
  );
}
