import { Input, Label, TextField } from "@heroui/react";
import { useState, type FormEvent } from "react";
import { useSaveBackupPolicy, type BackupPolicy } from "../../api/backups";
import { PrimaryButton } from "../ui/PrimaryButton";

const RETENTION_FIELDS = [
  { key: "keep_hourly", label: "Hourly" },
  { key: "keep_daily", label: "Daily" },
  { key: "keep_weekly", label: "Weekly" },
  { key: "keep_monthly", label: "Monthly" },
] as const;

type RetentionKey = (typeof RETENTION_FIELDS)[number]["key"];

interface Draft {
  retention: Record<RetentionKey, string>;
  excludes: string;
}

interface BackupPolicyEditorProps {
  instanceId: number;
  policy: BackupPolicy;
}

function toDraft(policy: BackupPolicy): Draft {
  return {
    retention: {
      keep_hourly: String(policy.keep_hourly),
      keep_daily: String(policy.keep_daily),
      keep_weekly: String(policy.keep_weekly),
      keep_monthly: String(policy.keep_monthly),
    },
    excludes: policy.excludes.join("\n"),
  };
}

function toPolicy(draft: Draft): BackupPolicy {
  const retention = Object.fromEntries(
    RETENTION_FIELDS.map(({ key }) => [key, Math.max(0, Math.floor(Number(draft.retention[key])) || 0)]),
  ) as Record<RetentionKey, number>;
  return {
    ...retention,
    excludes: draft.excludes
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean),
  };
}

function describeRetention(policy: BackupPolicy): string {
  const kept = RETENTION_FIELDS.filter(({ key }) => policy[key] > 0).map(({ key, label }) => `${policy[key]} ${label.toLowerCase()}`);
  if (kept.length === 0) return "Keeping every snapshot";
  return `Keeping ${kept.join(", ")} scheduled snapshots. Manual backups stay until you delete them`;
}

export function BackupPolicyEditor({ instanceId, policy }: BackupPolicyEditorProps) {
  const save = useSaveBackupPolicy(instanceId);
  const [draft, setDraft] = useState(() => toDraft(policy));

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    save.mutate(toPolicy(draft), { onSuccess: (saved) => setDraft(toDraft(saved)) });
  };

  return (
    <form onSubmit={submit} className="space-y-5">
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        {RETENTION_FIELDS.map(({ key, label }) => (
          <TextField
            key={key}
            value={draft.retention[key]}
            onChange={(value: string) => setDraft({ ...draft, retention: { ...draft.retention, [key]: value } })}
            className="flex flex-col gap-1.5"
          >
            <Label className="text-sm font-medium text-ink-2">{label}</Label>
            <Input type="number" min={0} inputMode="numeric" />
          </TextField>
        ))}
      </div>

      <div className="space-y-1.5">
        <label htmlFor="backup-excludes" className="text-sm font-medium text-ink-2">
          Exclude patterns
        </label>
        <textarea
          id="backup-excludes"
          rows={4}
          spellCheck={false}
          value={draft.excludes}
          onChange={(event) => setDraft({ ...draft, excludes: event.target.value })}
          placeholder={"*.log\ncache/\nlogs/"}
          className="w-full rounded-control border border-line bg-input p-3 font-mono text-sm text-ink-1 placeholder:text-ink-4 focus:border-accent"
        />
        <p className="text-xs text-ink-3">One .gitignore-style pattern per line. Excluded files are never backed up.</p>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="space-y-1">
          <p className="font-mono text-xs text-ink-3">{describeRetention(toPolicy(draft))}</p>
          {save.isError && <p className="text-sm text-rose">{save.error.message}</p>}
          {save.isSuccess && !save.isPending && <p className="text-sm text-green">Retention saved</p>}
        </div>
        <PrimaryButton type="submit" isDisabled={save.isPending}>
          Save policy
        </PrimaryButton>
      </div>
    </form>
  );
}
