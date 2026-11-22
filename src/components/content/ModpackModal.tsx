import { Checkbox, Input, Label, Modal, TextField } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Link } from "react-router";
import type { Project, ProjectVersion } from "../../api/content";
import { useInstallModpack } from "../../api/content";
import { instancesKey } from "../../api/instances";
import { useTasks, type TaskView } from "../../api/tasks";
import { ProgressBar } from "../ui/ProgressBar";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";

export interface ModpackAction {
  project: Project;
  version: ProjectVersion;
}

interface ModpackModalProps {
  action: ModpackAction | null;
  onClose: () => void;
}

interface ModpackResult {
  instance_id: number;
  installed: number;
  failed: { file: string; error: string }[];
}

export function ModpackModal({ action, onClose }: ModpackModalProps) {
  return (
    <Modal.Backdrop isOpen={action !== null} onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="md">
        <Modal.Dialog className="bg-panel text-ink-1">
          {action && <ModpackForm action={action} onClose={onClose} />}
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

function ModpackForm({ action, onClose }: { action: ModpackAction; onClose: () => void }) {
  const [name, setName] = useState(action.project.name);
  const [acceptEula, setAcceptEula] = useState(false);
  const install = useInstallModpack();
  const taskId = install.data?.task_id ?? null;
  const task = useTaskById(taskId);
  const queryClient = useQueryClient();
  const finished = task?.status === "finished";

  useEffect(() => {
    if (finished) void queryClient.invalidateQueries({ queryKey: instancesKey, exact: true });
  }, [finished, queryClient]);

  const submit = () =>
    install.mutate({
      platform: action.project.platform,
      projectId: action.project.id,
      versionId: action.version.id,
      name: name.trim() || null,
      acceptEula,
    });

  return (
    <>
      <Modal.Header className="flex items-start gap-3">
        <span className="grid size-10 place-items-center rounded-control bg-accent-tint text-accent">
          <Icon icon="lucide:package-plus" width={18} height={18} />
        </span>
        <div className="min-w-0 space-y-1">
          <Modal.Heading className="text-lg font-semibold text-ink-0">Create instance from modpack</Modal.Heading>
          <p className="truncate font-mono text-[11px] text-ink-3">
            {action.project.name} · {action.version.name || action.version.version_number}
          </p>
        </div>
      </Modal.Header>
      <Modal.Body className="space-y-5">
        {taskId === null ? (
          <>
            <TextField value={name} onChange={setName} isDisabled={install.isPending} className="flex flex-col gap-1.5">
              <Label className="text-sm font-medium text-ink-2">Instance name</Label>
              <Input placeholder={action.project.name} />
            </TextField>
            <Checkbox isSelected={acceptEula} onChange={setAcceptEula} isDisabled={install.isPending}>
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              <Checkbox.Content className="text-sm text-ink-1">
                I accept the Minecraft EULA (https://aka.ms/MinecraftEULA).
              </Checkbox.Content>
            </Checkbox>
            {install.error && <p className="text-sm text-rose">{install.error.message}</p>}
          </>
        ) : (
          <TaskProgress task={task} />
        )}
      </Modal.Body>
      <Modal.Footer className="flex justify-end gap-2">
        <GhostButton onPress={onClose}>{taskId === null ? "Cancel" : "Close"}</GhostButton>
        {taskId === null && (
          <PrimaryButton isDisabled={!acceptEula || install.isPending} onPress={submit}>
            {install.isPending ? "Starting…" : "Create instance"}
          </PrimaryButton>
        )}
      </Modal.Footer>
    </>
  );
}

function useTaskById(taskId: number | null): TaskView | undefined {
  const { data } = useTasks();
  return taskId === null ? undefined : data?.find((entry) => entry.id === taskId);
}

function TaskProgress({ task }: { task: TaskView | undefined }) {
  if (!task) return <p className="text-sm text-ink-3">Queued…</p>;

  if (task.status === "failed") {
    return <p className="text-sm text-rose">{task.error ?? "The modpack install failed."}</p>;
  }

  if (task.status === "finished") {
    const result = task.result as ModpackResult | null;
    const failed = result?.failed.length ?? 0;
    return (
      <div className="space-y-4">
        <p className="flex items-center gap-2 text-sm text-cyan">
          <Icon icon="lucide:circle-check" width={16} height={16} />
          Installed {result?.installed ?? 0} files{failed > 0 ? `, ${failed} need manual install` : ""}.
        </p>
        {result && failed > 0 && (
          <ul className="max-h-40 space-y-1 overflow-y-auto font-mono text-[11px] text-ink-3">
            {result.failed.map((entry) => (
              <li key={entry.file} className="truncate" title={entry.error}>
                {entry.file}: {entry.error}
              </li>
            ))}
          </ul>
        )}
        {result && (
          <Link to={`/instances/${result.instance_id}/content`} className="inline-flex items-center gap-2 text-sm font-medium text-accent hover:underline">
            Open instance
            <Icon icon="lucide:arrow-right" width={14} height={14} />
          </Link>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <ProgressBar value={task.progress * 100} />
      <p className="font-mono text-xs text-ink-3">{task.message || "Working…"}</p>
    </div>
  );
}
