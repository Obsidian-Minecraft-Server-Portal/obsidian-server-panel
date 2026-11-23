import { Input, Label, TextField } from "@heroui/react";
import { CheckboxRow } from "../ui/CheckboxRow";
import { useEffect, useRef, useState } from "react";
import { useExportServerPack, type AnalysisResult, type ServerPackReceipt } from "../../api/serverpack";
import { useTasks, type TaskView } from "../../api/tasks";
import { LOADER_META } from "../runtime/loaderMeta";
import { ProgressBar } from "../ui/ProgressBar";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { effectiveInclude } from "./model";
import type { Decisions } from "./ReviewStep";

interface ExportStepProps {
  draft: AnalysisResult;
  decisions: Decisions;
  onBack: () => void;
  onDone: (receipt: ServerPackReceipt) => void;
}

export function ExportStep({ draft, decisions, onBack, onDone }: ExportStepProps) {
  const [name, setName] = useState(draft.pack_name);
  const [acceptEula, setAcceptEula] = useState(false);
  const exportPack = useExportServerPack(draft.draft_id);
  const taskId = exportPack.data?.task_id ?? null;
  const task = useTaskById(taskId);
  const handled = useRef(false);
  const inPack = draft.mods.filter((mod) => effectiveInclude(mod, decisions.mods[mod.index])).length;
  const folders = draft.folders.filter((folder) => (decisions.folders[folder.name] ?? folder.include)).length;

  useEffect(() => {
    if (handled.current || task?.status !== "finished") return;
    const receipt = receiptOf(task);
    if (receipt === null) return;
    handled.current = true;
    onDone(receipt);
  }, [task, onDone]);

  const submit = () =>
    exportPack.mutate({
      name: name.trim() || draft.pack_name,
      accept_eula: acceptEula,
      mods: draft.mods.map((mod) => ({ index: mod.index, include: effectiveInclude(mod, decisions.mods[mod.index]) })),
      folders: draft.folders.map((folder) => ({
        name: folder.name,
        include: decisions.folders[folder.name] ?? folder.include,
      })),
    });

  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-[minmax(0,1fr)_16rem]">
      <div className="max-w-xl space-y-5">
        {taskId === null ? (
          <>
            <TextField value={name} onChange={setName} isDisabled={exportPack.isPending} className="flex flex-col gap-1.5">
              <Label className="text-sm font-medium text-ink-2">Server name</Label>
              <Input placeholder={draft.pack_name} />
            </TextField>
            <CheckboxRow isSelected={acceptEula} onChange={setAcceptEula} isDisabled={exportPack.isPending}>
              I accept the Minecraft EULA (https://aka.ms/MinecraftEULA).
            </CheckboxRow>
            {exportPack.error && <p className="text-sm text-rose">{exportPack.error.message}</p>}
            <div className="flex justify-end gap-2">
              <GhostButton onPress={onBack} isDisabled={exportPack.isPending}>
                Back
              </GhostButton>
              <PrimaryButton isDisabled={!acceptEula || exportPack.isPending} isPending={exportPack.isPending} onPress={submit}>
                Create server
              </PrimaryButton>
            </div>
          </>
        ) : (
          <ExportProgress task={task} />
        )}
      </div>
      <aside className="space-y-2 self-start rounded-panel border border-line bg-panel p-5 text-sm">
        <p className="font-medium text-ink-0">{draft.pack_name}</p>
        <p className="font-mono text-[11px] text-ink-3">
          {LOADER_META[draft.loader].label} {draft.mc_version}
        </p>
        <p className="text-ink-2">
          {inPack} mods · {folders} folders
        </p>
      </aside>
    </div>
  );
}

function useTaskById(taskId: number | null): TaskView | undefined {
  const { data } = useTasks();
  return taskId === null ? undefined : data?.find((entry) => entry.id === taskId);
}

function receiptOf(task: TaskView): ServerPackReceipt | null {
  const result = task.result;
  if (!result || typeof result.instance_id !== "number") return null;
  return {
    instance_id: result.instance_id,
    included: Number(result.included ?? 0),
    excluded: Number(result.excluded ?? 0),
    warnings: Array.isArray(result.warnings) ? result.warnings.map(String) : [],
  };
}

function ExportProgress({ task }: { task: TaskView | undefined }) {
  if (!task) return <p className="text-sm text-ink-3">Queued…</p>;
  if (task.status === "failed") return <p className="text-sm text-rose">{task.error ?? "The server could not be created."}</p>;
  return (
    <div className="space-y-3">
      <ProgressBar value={task.progress * 100} />
      <p className="font-mono text-xs text-ink-3">{task.message || "Creating server…"}</p>
    </div>
  );
}
