import { Input, Label, Modal, TextField } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Link, useNavigate } from "react-router";
import type { Project, ProjectVersion } from "../../api/content";
import { useInstallModpack } from "../../api/content";
import { instancesKey } from "../../api/instances";
import { useAnalyzeModpack } from "../../api/serverpack";
import { useTaskById, type TaskView } from "../../api/tasks";
import { CheckboxRow } from "../ui/CheckboxRow";
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
  warnings?: string[];
}

export function ModpackModal({ action, onClose }: ModpackModalProps) {
  return (
    <Modal.Backdrop isOpen={action !== null} onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="md">
        <Modal.Dialog className="bg-panel text-ink-1">
          {action &&
            (action.version.has_server_pack ? (
              <OfficialPackForm action={action} onClose={onClose} />
            ) : (
              <NoServerPackForm action={action} onClose={onClose} />
            ))}
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

function OfficialPackForm({ action, onClose }: { action: ModpackAction; onClose: () => void }) {
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
      source: "server_pack",
    });

  return (
    <>
      <Modal.Header className="flex items-start gap-3">
        <span className="grid size-10 place-items-center rounded-control bg-secondary-tint text-secondary">
          <Icon icon="lucide:package-plus" width={18} height={18} />
        </span>
        <div className="min-w-0 space-y-1">
          <Modal.Heading className="text-lg font-semibold text-ink-0">Create server from server pack</Modal.Heading>
          <p className="truncate font-mono text-[11px] text-ink-3">
            {action.project.name} · {action.version.name || action.version.version_number}
          </p>
        </div>
      </Modal.Header>
      <Modal.Body className="space-y-5">
        {taskId === null ? (
          <>
            <p className="flex items-start gap-2 rounded-control border border-line bg-input px-3 py-2.5 text-xs text-ink-2">
              <Icon icon="lucide:info" width={14} height={14} className="mt-px shrink-0 text-secondary" />
              This version lists an official server pack, so its server files are installed instead of the client files.
            </p>
            <TextField value={name} onChange={setName} isDisabled={install.isPending} className="flex flex-col gap-1.5">
              <Label className="text-sm font-medium text-ink-2">Instance name</Label>
              <Input placeholder={action.project.name} />
            </TextField>
            <CheckboxRow isSelected={acceptEula} onChange={setAcceptEula} isDisabled={install.isPending}>
              I accept the Minecraft EULA (https://aka.ms/MinecraftEULA).
            </CheckboxRow>
            {install.error && <p className="text-sm text-rose">{install.error.message}</p>}
          </>
        ) : (
          <InstallProgress task={task} />
        )}
      </Modal.Body>
      <Modal.Footer className="flex justify-end gap-2">
        <GhostButton onPress={onClose}>{taskId === null ? "Cancel" : "Close"}</GhostButton>
        {taskId === null && (
          <PrimaryButton isDisabled={!acceptEula || install.isPending} onPress={submit}>
            {install.isPending ? "Starting…" : "Create server"}
          </PrimaryButton>
        )}
      </Modal.Footer>
    </>
  );
}

function NoServerPackForm({ action, onClose }: { action: ModpackAction; onClose: () => void }) {
  const analyze = useAnalyzeModpack();
  const navigate = useNavigate();
  const taskId = analyze.data?.task_id ?? null;
  const task = useTaskById(taskId);
  const draftId = task?.status === "finished" ? draftIdOf(task) : null;
  const navigated = useRef(false);

  useEffect(() => {
    if (draftId === null || navigated.current) return;
    navigated.current = true;
    onClose();
    navigate(`/discover/server-pack/${draftId}`);
  }, [draftId, navigate, onClose]);

  const failure =
    task?.status === "failed" ? (task.error ?? "The modpack could not be analyzed.") : (analyze.error?.message ?? null);

  return (
    <>
      <Modal.Header className="flex items-start gap-3">
        <span className="grid size-10 place-items-center rounded-control bg-secondary-tint text-secondary">
          <Icon icon="lucide:hammer" width={18} height={18} />
        </span>
        <div className="min-w-0 space-y-1">
          <Modal.Heading className="text-lg font-semibold text-ink-0">No official server pack</Modal.Heading>
          <p className="truncate font-mono text-[11px] text-ink-3">
            {action.project.name} · {action.version.name || action.version.version_number}
          </p>
        </div>
      </Modal.Header>
      <Modal.Body className="space-y-4">
        <p className="flex items-start gap-2 rounded-control border border-amber/40 bg-input px-3 py-2.5 text-sm text-ink-2">
          <Icon icon="lucide:triangle-alert" width={16} height={16} className="mt-px shrink-0 text-amber" />
          This version does not list a server pack. Client modpacks include mods that crash dedicated servers. Build a server
          pack to review every mod and leave the client-only ones out.
        </p>
        {taskId !== null && <AnalysisProgress task={task} />}
        {failure && <p className="text-sm text-rose">{failure}</p>}
      </Modal.Body>
      <Modal.Footer className="flex justify-end gap-2">
        <GhostButton onPress={onClose}>{taskId === null ? "Cancel" : "Close"}</GhostButton>
        {taskId === null && (
          <PrimaryButton
            isPending={analyze.isPending}
            onPress={() =>
              analyze.mutate({
                platform: action.project.platform,
                projectId: action.project.id,
                versionId: action.version.id,
              })
            }
          >
            Build server pack
          </PrimaryButton>
        )}
      </Modal.Footer>
    </>
  );
}

function draftIdOf(task: TaskView): string | null {
  return (task.result as { draft_id?: string } | null)?.draft_id ?? null;
}

function AnalysisProgress({ task }: { task: TaskView | undefined }) {
  if (!task) return <p className="text-sm text-ink-3">Queued…</p>;
  if (task.status === "failed" || task.status === "finished") return null;
  return (
    <div className="space-y-3">
      <ProgressBar value={task.progress * 100} />
      <p className="font-mono text-xs text-ink-3">{task.message || "Analyzing mods…"}</p>
    </div>
  );
}

function InstallProgress({ task }: { task: TaskView | undefined }) {
  if (!task) return <p className="text-sm text-ink-3">Queued…</p>;

  if (task.status === "failed") {
    return <p className="text-sm text-rose">{task.error ?? "The modpack install failed."}</p>;
  }

  if (task.status === "finished") {
    const result = task.result as ModpackResult | null;
    const failed = result?.failed.length ?? 0;
    const warnings = result?.warnings ?? [];
    return (
      <div className="space-y-4">
        <p className="flex items-center gap-2 text-sm text-green">
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
        {warnings.length > 0 && (
          <ul className="space-y-1 text-xs text-amber">
            {warnings.map((warning) => (
              <li key={warning}>{warning}</li>
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
