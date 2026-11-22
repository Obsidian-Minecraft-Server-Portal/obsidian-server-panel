import { Input, Label, TextField } from "@heroui/react";
import { toInputDateTime, toInputTime, toWireDateTime, toWireTime } from "./model";

interface FieldProps {
  label: string;
  className?: string;
}

interface NumberInputProps extends FieldProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
}

export function NumberInput({ label, value, onChange, min = 0, max, className }: NumberInputProps) {
  return (
    <TextField
      value={String(value)}
      onChange={(text: string) => onChange(Math.max(0, Math.floor(Number(text)) || 0))}
      className={className ?? "flex min-w-28 flex-col gap-1.5"}
    >
      <Label className="text-sm font-medium text-ink-2">{label}</Label>
      <Input type="number" min={min} max={max} inputMode="numeric" />
    </TextField>
  );
}

interface WireInputProps extends FieldProps {
  value: string;
  onChange: (wire: string) => void;
}

export function TimeInput({ label, value, onChange, className }: WireInputProps) {
  return (
    <TextField
      value={toInputTime(value)}
      onChange={(input: string) => onChange(toWireTime(input))}
      className={className ?? "flex min-w-32 flex-col gap-1.5"}
    >
      <Label className="text-sm font-medium text-ink-2">{label}</Label>
      <Input type="time" step={60} />
    </TextField>
  );
}

export function DateTimeInput({ label, value, onChange, className }: WireInputProps) {
  return (
    <TextField
      value={toInputDateTime(value)}
      onChange={(input: string) => onChange(toWireDateTime(input))}
      className={className ?? "flex min-w-52 flex-col gap-1.5"}
    >
      <Label className="text-sm font-medium text-ink-2">{label}</Label>
      <Input type="datetime-local" />
    </TextField>
  );
}
