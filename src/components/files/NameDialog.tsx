import { Modal } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState, type FormEvent } from "react";
import { FormField } from "../instance/FormField";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { errorMessage } from "./fileUtils";

interface NameDialogProps {
  title: string;
  icon: string;
  label: string;
  initialValue: string;
  submitLabel: string;
  validate: (value: string) => string | null;
  onSubmit: (value: string) => Promise<void>;
  onClose: () => void;
}

export function NameDialog({ title, icon, label, initialValue, submitLabel, validate, onSubmit, onClose }: NameDialogProps) {
  const [value, setValue] = useState(initialValue);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const trimmed = value.trim();
  const problem = validate(trimmed);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (problem || busy) return;
    setBusy(true);
    setError(null);
    try {
      await onSubmit(trimmed);
      onClose();
    } catch (failure) {
      setError(errorMessage(failure));
      setBusy(false);
    }
  };

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
              <FormField label={label} value={value} onChange={setValue} />
              {(error || (value && problem)) && <p className="text-sm text-rose">{error ?? problem}</p>}
            </Modal.Body>
            <Modal.Footer className="flex justify-end gap-2">
              <GhostButton onPress={onClose}>Cancel</GhostButton>
              <PrimaryButton type="submit" isDisabled={!!problem || busy}>
                {busy ? "Working…" : submitLabel}
              </PrimaryButton>
            </Modal.Footer>
          </form>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}
