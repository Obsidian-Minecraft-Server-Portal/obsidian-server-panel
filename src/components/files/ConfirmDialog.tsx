import { Modal } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState, type ReactNode } from "react";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { errorMessage } from "./fileUtils";

interface ConfirmDialogProps {
  title: string;
  description: ReactNode;
  confirmLabel: string;
  danger?: boolean;
  onConfirm: () => Promise<void>;
  onClose: () => void;
}

export function ConfirmDialog({ title, description, confirmLabel, danger, onConfirm, onClose }: ConfirmDialogProps) {
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const confirm = async () => {
    setBusy(true);
    setError(null);
    try {
      await onConfirm();
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
          <Modal.Header className="flex items-center gap-3">
            <span
              className={`grid size-10 place-items-center rounded-control ${danger ? "bg-rose/12 text-rose" : "bg-accent-tint text-accent"}`}
            >
              <Icon icon={danger ? "lucide:triangle-alert" : "lucide:circle-help"} width={18} height={18} />
            </span>
            <Modal.Heading className="text-lg font-semibold text-ink-0">{title}</Modal.Heading>
          </Modal.Header>
          <Modal.Body className="space-y-3 text-sm text-ink-2">
            {description}
            {error && <p className="text-rose">{error}</p>}
          </Modal.Body>
          <Modal.Footer className="flex justify-end gap-2">
            <GhostButton onPress={onClose}>Cancel</GhostButton>
            <PrimaryButton isDisabled={busy} onPress={confirm} className={danger ? "from-rose to-rose/80 text-ink-0" : undefined}>
              {busy ? "Working…" : confirmLabel}
            </PrimaryButton>
          </Modal.Footer>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}
