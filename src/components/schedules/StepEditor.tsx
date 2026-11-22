import { Button, Checkbox, Input, Label, TextField } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useEffect, useState } from "react";
import type { Action, Step } from "../../api/schedules";
import { OptionSelect } from "../runtime/OptionSelect";
import { GhostButton } from "../ui/GhostButton";
import { NumberInput } from "./FormFields";
import { ACTION_OPTIONS, newStep, type ActionKind } from "./model";

interface StepEditorProps {
  steps: Step[];
  onChange: (steps: Step[]) => void;
}

export function StepEditor({ steps, onChange }: StepEditorProps) {
  const replace = (index: number, step: Step) => onChange(steps.map((current, i) => (i === index ? step : current)));
  const remove = (index: number) => onChange(steps.filter((_, i) => i !== index));
  const move = (index: number, delta: number) => {
    const target = index + delta;
    if (target < 0 || target >= steps.length) return;
    const next = [...steps];
    [next[index], next[target]] = [next[target], next[index]];
    onChange(next);
  };

  return (
    <div className="space-y-3">
      <ol className="space-y-3">
        {steps.map((step, index) => (
          <StepRow
            key={index}
            index={index}
            isFirst={index === 0}
            isLast={index === steps.length - 1}
            step={step}
            onChange={(next) => replace(index, next)}
            onRemove={() => remove(index)}
            onMove={(delta) => move(index, delta)}
          />
        ))}
      </ol>
      <GhostButton size="sm" onPress={() => onChange([...steps, newStep("backup")])}>
        <Icon icon="lucide:plus" width={14} height={14} />
        Add step
      </GhostButton>
    </div>
  );
}

interface StepRowProps {
  step: Step;
  index: number;
  isFirst: boolean;
  isLast: boolean;
  onChange: (step: Step) => void;
  onRemove: () => void;
  onMove: (delta: number) => void;
}

function StepRow({ step, index, isFirst, isLast, onChange, onRemove, onMove }: StepRowProps) {
  return (
    <li className="space-y-3 rounded-control border border-line bg-input/60 p-3">
      <div className="flex flex-wrap items-end gap-2">
        <span className="pb-2 font-mono text-xs text-ink-3">{index + 1}</span>
        <OptionSelect
          label="Action"
          options={ACTION_OPTIONS}
          value={step.action.type}
          onChange={(type) => type && onChange({ ...step, action: newStep(type as ActionKind).action })}
          className="min-w-52 flex-1"
        />
        <div className="flex items-center gap-1 pb-0.5">
          <IconAction icon="lucide:arrow-up" label="Move step up" isDisabled={isFirst} onPress={() => onMove(-1)} />
          <IconAction icon="lucide:arrow-down" label="Move step down" isDisabled={isLast} onPress={() => onMove(1)} />
          <IconAction icon="lucide:trash-2" label="Remove step" onPress={onRemove} />
        </div>
      </div>

      <ActionFields action={step.action} onChange={(action) => onChange({ ...step, action })} />

      <div className="flex flex-wrap items-end gap-4">
        <NumberInput
          label="Delay (seconds)"
          value={step.delay_secs}
          max={86_400}
          onChange={(delay_secs) => onChange({ ...step, delay_secs })}
        />
        <Checkbox
          isSelected={step.only_if_running}
          onChange={(only_if_running: boolean) => onChange({ ...step, only_if_running })}
        >
          <Checkbox.Control>
            <Checkbox.Indicator />
          </Checkbox.Control>
          <Checkbox.Content className="text-sm text-ink-2">Only if the server is running</Checkbox.Content>
        </Checkbox>
      </div>
    </li>
  );
}

interface IconActionProps {
  icon: string;
  label: string;
  isDisabled?: boolean;
  onPress: () => void;
}

function IconAction({ icon, label, isDisabled, onPress }: IconActionProps) {
  return (
    <Button
      isIconOnly
      size="sm"
      variant="ghost"
      aria-label={label}
      isDisabled={isDisabled}
      onPress={onPress}
      className="text-ink-3 hover:text-ink-0"
    >
      <Icon icon={icon} width={16} height={16} />
    </Button>
  );
}

interface ActionFieldsProps {
  action: Action;
  onChange: (action: Action) => void;
}

function ActionFields({ action, onChange }: ActionFieldsProps) {
  switch (action.type) {
    case "command":
      return (
        <TextField value={action.line} onChange={(line: string) => onChange({ type: "command", line })} className="flex flex-col gap-1.5">
          <Label className="text-sm font-medium text-ink-2">Console command</Label>
          <Input placeholder="say Restarting soon" spellCheck={false} className="font-mono" />
        </TextField>
      );
    case "restart":
      return (
        <CountdownInput
          countdown={action.countdown_secs}
          onChange={(countdown_secs) => onChange({ type: "restart", countdown_secs })}
        />
      );
    default:
      return null;
  }
}

function parseCountdown(text: string): number[] {
  return text
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean)
    .map(Number)
    .filter((secs) => Number.isInteger(secs) && secs >= 0);
}

interface CountdownInputProps {
  countdown: number[];
  onChange: (countdown: number[]) => void;
}

function CountdownInput({ countdown, onChange }: CountdownInputProps) {
  const [text, setText] = useState(countdown.join(", "));

  useEffect(() => {
    setText((current) => (parseCountdown(current).join() === countdown.join() ? current : countdown.join(", ")));
  }, [countdown]);

  return (
    <TextField
      value={text}
      onChange={(next: string) => {
        setText(next);
        onChange(parseCountdown(next));
      }}
      className="flex flex-col gap-1.5"
    >
      <Label className="text-sm font-medium text-ink-2">Countdown announcements (seconds, comma separated)</Label>
      <Input placeholder="300, 60, 10" inputMode="numeric" className="font-mono" />
    </TextField>
  );
}
