import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { JAVA_COMPONENTS, javaKey, useAvailableJava, useInstallJava } from "../../api/java";
import { useTasks } from "../../api/tasks";
import { Panel } from "../ui/Panel";
import { PrimaryButton } from "../ui/PrimaryButton";
import { ProgressBar } from "../ui/ProgressBar";
import { OptionSelect } from "./OptionSelect";

const DEFAULT_COMPONENT = "java-runtime-delta";

export function InstallRuntimePanel() {
  const queryClient = useQueryClient();
  const tasks = useTasks();
  const install = useInstallJava();
  const available = useAvailableJava();
  const options = JAVA_COMPONENTS.map((id) => {
    const runtime = available.data?.find((entry) => entry.component === id);
    const name = id.replace(/^java-runtime-|^jre-/, "");
    if (!runtime) return { id, label: name };
    return { id, label: `Java ${runtime.version} · ${name}${runtime.installed ? " · installed" : ""}` };
  });
  const [component, setComponent] = useState(DEFAULT_COMPONENT);
  const [taskId, setTaskId] = useState<number | null>(null);

  const task = tasks.data?.find((candidate) => candidate.id === taskId);
  const running = task?.status === "running";
  const finished = task?.status === "finished";

  useEffect(() => {
    if (finished) void queryClient.invalidateQueries({ queryKey: javaKey });
  }, [finished, queryClient]);

  return (
    <Panel title="Install runtime">
      <div className="space-y-5">
        <div className="flex flex-wrap items-end gap-4">
          <OptionSelect
            label="Managed runtime"
            className="min-w-64 flex-1"
            options={options}
            value={component}
            onChange={(next) => next && setComponent(next)}
            isDisabled={running || install.isPending}
          />
          <PrimaryButton
            isPending={install.isPending}
            isDisabled={running}
            onPress={() =>
              install.mutate(component, { onSuccess: ({ task_id }) => setTaskId(task_id) })
            }
          >
            Install
          </PrimaryButton>
        </div>

        {task && (
          <div className="space-y-2">
            <div className="flex items-center justify-between gap-3 font-mono text-xs text-ink-3">
              <span className="truncate">{task.message || task.title}</span>
              <span>{running ? `${Math.round(task.progress * 100)}%` : task.status.toUpperCase()}</span>
            </div>
            {running && <ProgressBar value={task.progress * 100} />}
            {task.status === "failed" && <p className="text-sm text-rose">{task.error ?? "Install failed"}</p>}
          </div>
        )}
        {install.error && <p className="text-sm text-rose">{install.error.message}</p>}
      </div>
    </Panel>
  );
}
