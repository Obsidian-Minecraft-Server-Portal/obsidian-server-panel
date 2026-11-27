import { Modal } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState, type FormEvent } from "react";
import { useTaskById, type TaskView } from "../../api/tasks";
import { FormField } from "../instance/FormField";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { ProgressBar } from "../ui/ProgressBar";
import { errorMessage } from "./fileUtils";

interface NameDialogProps {
  title: string;
  icon: string;
  label: string;
  initialValue: string;
  submitLabel: string;
  validate: (value: string) => string | null;
  /** Resolves with a background task id to follow its progress, or with nothing to close the dialog. */
  onSubmit: (value: string) => Promise<number | void>;
  onClose: () => void;
}

export function NameDialog({ title, icon, label, initialValue, submitLabel, validate, onSubmit, onClose }: NameDialogProps) {
  const [value, setValue] = useState(initialValue);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [taskId, setTaskId] = useState<number | null>(null);
  const trimmed = value.trim();
  const problem = busy ? null : validate(trimmed);
  const following = taskId !== null;
  const task = useTaskById(taskId);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (following || problem || busy) return;
    setBusy(true);
    setError(null);
    try {
      const started = await onSubmit(trimmed);
      if (typeof started === "number") {
        setTaskId(started);
        setBusy(false);
      } else {
        onClose();
      }
    } catch (failure) {
      setError(errorMessage(failure));
      setBusy(false);
    }
  };

  const message = error ?? (value ? problem : null);

  return (
    <Modal.Backdrop isOpen onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="md">
        <Modal.Dialog className="bg-panel text-ink-1">
          <form onSubmit={submit}>
            <Modal.Header className="flex items-center gap-3">
              <span className="grid size-10 place-items-center rounded-control bg-secondary-tint text-secondary">
                <Icon icon={icon} width={18} height={18} />
              </span>
              <Modal.Heading className="text-lg font-semibold text-ink-0">{title}</Modal.Heading>
            </Modal.Header>
            <Modal.Body className="space-y-3">
              {following ? (
                <TaskProgress task={task} />
              ) : (
                <>
                  <FormField label={label} value={value} onChange={setValue} />
                  {message && <p className="text-sm text-rose">{message}</p>}
                </>
              )}
            </Modal.Body>
            <Modal.Footer className="flex justify-end gap-2">
              {following ? (
                <GhostButton onPress={onClose}>Close</GhostButton>
              ) : (
                <>
                  <GhostButton onPress={onClose}>Cancel</GhostButton>
                  <PrimaryButton type="submit" isDisabled={!!problem || busy}>
                    {busy ? "Working…" : submitLabel}
                  </PrimaryButton>
                </>
              )}
            </Modal.Footer>
          </form>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

function TaskProgress({ task }: { task: TaskView | undefined }) {
  if (!task) return <p className="text-sm text-ink-3">Queued…</p>;
  if (task.status === "failed") {
    return <p className="text-sm text-rose">{task.error ?? "The archive could not be written."}</p>;
  }
  if (task.status === "finished") {
    const path = typeof task.result?.path === "string" ? task.result.path : "the archive";
    return (
      <p className="flex items-center gap-2 text-sm text-green">
        <Icon icon="lucide:circle-check" width={16} height={16} />
        Saved {path}.
      </p>
    );
  }
  return (
    <div className="space-y-3">
      <ProgressBar value={task.progress * 100} />
      <div className="flex items-center justify-between gap-3 font-mono text-xs text-ink-3">
        <span className="truncate">{task.message || "Compressing…"}</span>
        <span className="shrink-0">{Math.floor(task.progress * 100)}%</span>
      </div>
    </div>
  );
}
