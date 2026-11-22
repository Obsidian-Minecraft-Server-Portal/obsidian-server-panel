import { Switch } from "@heroui/react";
import { useState } from "react";
import type { ScheduleRecord } from "../../api/schedules";
import { formatDateTime } from "../backups/format";
import { ConfirmButton } from "../backups/ConfirmButton";
import { Chip } from "../ui/Chip";
import { GhostButton } from "../ui/GhostButton";
import { RunHistory } from "./RunHistory";
import { describeStep } from "./model";

interface ScheduleCardProps {
  record: ScheduleRecord;
  busy: boolean;
  onToggle: (enabled: boolean) => void;
  onRun: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function ScheduleCard({ record, busy, onToggle, onRun, onEdit, onDelete }: ScheduleCardProps) {
  const [showHistory, setShowHistory] = useState(false);
  const { schedule } = record;

  return (
    <article className="space-y-3 rounded-panel border border-line bg-raised/30 p-4">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <h3 className="truncate text-base font-semibold text-ink-0">{schedule.name}</h3>
          <p className="text-sm text-ink-2">{record.description}</p>
          <p className="font-mono text-[11px] text-ink-3">
            {record.next_run_at ? `NEXT ${formatDateTime(record.next_run_at)}` : "NO UPCOMING RUN"}
          </p>
        </div>
        <Switch isSelected={schedule.enabled} isDisabled={busy} onChange={onToggle} aria-label="Enabled">
          <Switch.Control>
            <Switch.Thumb />
          </Switch.Control>
        </Switch>
      </header>

      <div className="flex flex-wrap gap-1.5">
        {schedule.steps.map((step, index) => (
          <Chip key={index}>{describeStep(step)}</Chip>
        ))}
      </div>

      <div className="flex flex-wrap gap-1.5">
        <GhostButton size="sm" isDisabled={busy} onPress={onRun}>
          Run now
        </GhostButton>
        <GhostButton size="sm" isDisabled={busy} onPress={onEdit}>
          Edit
        </GhostButton>
        <GhostButton size="sm" onPress={() => setShowHistory((open) => !open)}>
          {showHistory ? "Hide history" : "History"}
        </GhostButton>
        <ConfirmButton label="Delete" confirmLabel="Confirm delete" danger isDisabled={busy} onConfirm={onDelete} />
      </div>

      {showHistory && <RunHistory scheduleId={record.id} />}
    </article>
  );
}
