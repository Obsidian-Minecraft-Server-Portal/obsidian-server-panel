import type { CompatibilityEntry, MigrationAction, ScanResponse } from "../../api/migration";
import { CheckboxRow } from "../ui/CheckboxRow";
import { GhostButton } from "../ui/GhostButton";
import { Panel } from "../ui/Panel";
import { PrimaryButton } from "../ui/PrimaryButton";

interface ReviewStepProps {
  scan: ScanResponse;
  decisions: Record<number, MigrationAction>;
  disableUnavailable: boolean;
  backup: boolean;
  onBackup: (value: boolean) => void;
  canBackup: boolean;
  acknowledged: boolean;
  onAcknowledge: (value: boolean) => void;
  isPending: boolean;
  blocked: boolean;
  error: string | null;
  onBack: () => void;
  onStart: () => void;
}

function nameOf(entry: CompatibilityEntry): string {
  return entry.item.name ?? entry.item.filename;
}

export function ReviewStep({
  scan,
  decisions,
  disableUnavailable,
  backup,
  onBackup,
  canBackup,
  acknowledged,
  onAcknowledge,
  isPending,
  blocked,
  error,
  onBack,
  onStart,
}: ReviewStepProps) {
  const { entries, current, target } = scan.report;
  const upgrades = entries.flatMap((entry) =>
    entry.compatibility.status === "upgradable" ? [{ entry, version: entry.compatibility.version.version_number }] : [],
  );
  const dependencies = new Map<string, string>();
  for (const entry of entries) {
    if (entry.compatibility.status !== "upgradable") continue;
    for (const dependency of entry.compatibility.dependencies) {
      dependencies.set(dependency.project_id, dependency.version.name);
    }
  }
  const disabling = entries.filter((entry) => {
    if (entry.compatibility.status === "unavailable") return disableUnavailable;
    if (entry.compatibility.status === "needs_decision") return decisions[entry.item.id] === "disable";
    return false;
  });
  const javaChange = current.mc_version !== target.mc_version;
  const canStart = (backup || acknowledged) && !isPending && !blocked;

  return (
    <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_22rem]">
      <Panel title="What will change">
        <div className="space-y-5 text-sm">
          <section className="space-y-1">
            <h3 className="font-semibold text-ink-0">Server software</h3>
            <p className="font-mono text-xs text-ink-2">
              Minecraft {current.mc_version} → {target.mc_version}
              {target.loader_version ? ` · loader ${target.loader_version}` : ""}
            </p>
            {javaChange && <p className="text-xs text-ink-3">The Java runtime is chosen again for the new version.</p>}
          </section>

          <section className="space-y-2">
            <h3 className="font-semibold text-ink-0">Upgrade ({upgrades.length})</h3>
            {upgrades.length === 0 ? (
              <p className="text-xs text-ink-3">No mods or plugins are upgraded.</p>
            ) : (
              <ul className="space-y-1">
                {upgrades.map(({ entry, version }) => (
                  <li key={entry.item.id} className="flex justify-between gap-3">
                    <span className="min-w-0 truncate text-ink-1">{nameOf(entry)}</span>
                    <span className="shrink-0 font-mono text-xs text-ink-3">{version}</span>
                  </li>
                ))}
              </ul>
            )}
          </section>

          {dependencies.size > 0 && (
            <section className="space-y-2">
              <h3 className="font-semibold text-ink-0">Dependencies added ({dependencies.size})</h3>
              <ul className="space-y-1">
                {[...dependencies.values()].map((name) => (
                  <li key={name} className="truncate text-ink-1">
                    {name}
                  </li>
                ))}
              </ul>
            </section>
          )}

          <section className="space-y-2">
            <h3 className="font-semibold text-ink-0">Disable ({disabling.length})</h3>
            {disabling.length === 0 ? (
              <p className="text-xs text-ink-3">Nothing is disabled. Disabled mods stay on disk and can be re-enabled.</p>
            ) : (
              <ul className="space-y-1">
                {disabling.map((entry) => (
                  <li key={entry.item.id} className="truncate text-ink-2">
                    {nameOf(entry)}
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      </Panel>

      <Panel title="Backup and start">
        <div className="space-y-5">
          <CheckboxRow isSelected={backup} onChange={onBackup} isDisabled={!canBackup || isPending}>
            Back up the server before upgrading
          </CheckboxRow>
          {backup ? (
            <p className="text-xs text-ink-3">
              The snapshot lets you roll back to the current worlds, mods and server jar if the upgrade fails.
            </p>
          ) : (
            <div className="space-y-3 rounded-control border border-amber/40 bg-input p-4">
              <p className="text-xs text-amber">
                Without a backup, a failed upgrade or a world upgraded by Minecraft cannot be undone.
              </p>
              <CheckboxRow isSelected={acknowledged} onChange={onAcknowledge} isDisabled={isPending}>
                I understand that this upgrade cannot be rolled back
              </CheckboxRow>
            </div>
          )}
          {error && <p className="text-sm text-rose">{error}</p>}
          <div className="flex flex-wrap justify-between gap-2">
            <GhostButton onPress={onBack} isDisabled={isPending}>
              Back
            </GhostButton>
            <PrimaryButton isDisabled={!canStart} isPending={isPending} onPress={onStart}>
              Start migration
            </PrimaryButton>
          </div>
        </div>
      </Panel>
    </div>
  );
}
