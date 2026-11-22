import { cn } from "@heroui/react";
import { useEffect, useState } from "react";
import { GhostButton } from "../ui/GhostButton";

const ARM_TIMEOUT_MS = 4_000;

interface ConfirmButtonProps {
  label: string;
  confirmLabel: string;
  onConfirm: () => void;
  danger?: boolean;
  isDisabled?: boolean;
}

export function ConfirmButton({ label, confirmLabel, onConfirm, danger, isDisabled }: ConfirmButtonProps) {
  const [armed, setArmed] = useState(false);

  useEffect(() => {
    if (!armed) return;
    const timer = window.setTimeout(() => setArmed(false), ARM_TIMEOUT_MS);
    return () => window.clearTimeout(timer);
  }, [armed]);

  return (
    <GhostButton
      size="sm"
      isDisabled={isDisabled}
      className={cn(armed && (danger ? "border-rose text-rose" : "border-accent text-[#d9a8ff]"))}
      onPress={() => {
        if (!armed) return setArmed(true);
        setArmed(false);
        onConfirm();
      }}
    >
      {armed ? confirmLabel : label}
    </GhostButton>
  );
}
