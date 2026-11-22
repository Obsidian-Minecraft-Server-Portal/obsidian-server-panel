import { Description, FieldError, Input, Label, TextField, cn } from "@heroui/react";

interface FormFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: "text" | "email" | "password" | "url";
  placeholder?: string;
  autoComplete?: string;
  autoFocus?: boolean;
  inputMode?: "text" | "numeric" | "email" | "url";
  maxLength?: number;
  description?: string;
  error?: string;
  isRequired?: boolean;
  isDisabled?: boolean;
  className?: string;
}

export function FormField({
  label,
  value,
  onChange,
  type = "text",
  placeholder,
  autoComplete,
  autoFocus,
  inputMode,
  maxLength,
  description,
  error,
  isRequired,
  isDisabled,
  className,
}: FormFieldProps) {
  return (
    <TextField
      value={value}
      onChange={onChange}
      isRequired={isRequired}
      isDisabled={isDisabled}
      isInvalid={Boolean(error)}
      className={cn("w-full", className)}
    >
      <Label className="text-sm font-medium text-ink-2">{label}</Label>
      <Input
        type={type}
        placeholder={placeholder}
        autoComplete={autoComplete}
        autoFocus={autoFocus}
        inputMode={inputMode}
        maxLength={maxLength}
        className="rounded-control border border-line bg-input text-ink-0 placeholder:text-ink-4 focus:border-accent"
      />
      {error ? (
        <FieldError>{error}</FieldError>
      ) : (
        description && <Description className="text-xs text-ink-4">{description}</Description>
      )}
    </TextField>
  );
}
