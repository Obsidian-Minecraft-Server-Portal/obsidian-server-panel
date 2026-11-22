import { Switch, cn } from "@heroui/react";
import { useId, type ReactNode } from "react";

interface SwitchRowProps {
  label: string;
  description?: ReactNode;
  isSelected: boolean;
  onChange: (selected: boolean) => void;
  isDisabled?: boolean;
  className?: string;
}

export function SwitchRow({ label, description, isSelected, onChange, isDisabled, className }: SwitchRowProps) {
  const id = useId();
  return (
    <div
      className={cn(
        "flex items-center justify-between gap-6 py-3.5",
        isDisabled ? "opacity-60" : "cursor-pointer",
        className,
      )}
      onClick={(event) => {
        if (isDisabled || (event.target as HTMLElement).closest("label")) return;
        onChange(!isSelected);
      }}
    >
      <div className="min-w-0 space-y-0.5">
        <p id={`${id}-label`} className="text-sm font-medium text-ink-1">
          {label}
        </p>
        {description && (
          <p id={`${id}-description`} className="text-xs leading-relaxed text-ink-3">
            {description}
          </p>
        )}
      </div>
      <Switch
        isSelected={isSelected}
        onChange={onChange}
        isDisabled={isDisabled}
        aria-labelledby={`${id}-label`}
        aria-describedby={description ? `${id}-description` : undefined}
        className="shrink-0"
      >
        <Switch.Control>
          <Switch.Thumb />
        </Switch.Control>
      </Switch>
    </div>
  );
}

export function SwitchGroup({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div className={cn("divide-y divide-line rounded-panel border border-line bg-input/40 px-4", className)}>{children}</div>
  );
}
