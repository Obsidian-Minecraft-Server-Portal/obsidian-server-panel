import { Input, Label, TextArea, TextField, cn } from "@heroui/react";
import type { ReactNode } from "react";

const FIELD = "w-full rounded-control border border-line bg-input px-3 text-sm text-ink-1 placeholder:text-ink-4 focus:border-accent";

interface FormFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  hint?: ReactNode;
  type?: "text" | "number";
  placeholder?: string;
  isDisabled?: boolean;
  multiline?: boolean;
  rows?: number;
  required?: boolean;
  className?: string;
}

export function FormField({
  label,
  value,
  onChange,
  hint,
  type = "text",
  placeholder,
  isDisabled,
  multiline,
  rows = 4,
  required,
  className,
}: FormFieldProps) {
  return (
    <TextField value={value} onChange={onChange} isDisabled={isDisabled} className={cn("flex flex-col gap-1.5", className)}>
      <Label className="text-sm font-medium text-ink-2">
        {label}
        {required && <span className="ml-0.5 text-rose">*</span>}
      </Label>
      {multiline ? (
        <TextArea rows={rows} className={cn(FIELD, "h-auto py-2 font-mono")} />
      ) : (
        <Input type={type} placeholder={placeholder} className={cn(FIELD, "h-10")} />
      )}
      {hint && <p className="text-xs text-ink-3">{hint}</p>}
    </TextField>
  );
}

