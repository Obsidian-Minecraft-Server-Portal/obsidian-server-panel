import { Checkbox, cn } from "@heroui/react";
import type { ReactNode } from "react";

interface CheckboxRowProps {
  children: ReactNode;
  isSelected: boolean;
  onChange: (selected: boolean) => void;
  isDisabled?: boolean;
  className?: string;
}

export function CheckboxRow({ children, isSelected, onChange, isDisabled, className }: CheckboxRowProps) {
  return (
    <Checkbox
      isSelected={isSelected}
      onChange={onChange}
      isDisabled={isDisabled}
      className={cn("!flex !flex-row !items-start w-full gap-3", isDisabled ? "opacity-60" : "cursor-pointer", className)}
    >
      <Checkbox.Control className="mt-0.5 size-4 shrink-0 rounded-[5px] border border-ink-4 bg-input data-[selected=true]:border-accent data-[selected=true]:bg-accent">
        <Checkbox.Indicator />
      </Checkbox.Control>
      <Checkbox.Content className="min-w-0 flex-1 text-sm leading-snug text-ink-1">{children}</Checkbox.Content>
    </Checkbox>
  );
}
