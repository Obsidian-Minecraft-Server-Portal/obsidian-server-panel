import { Button } from "@heroui/react";
import type { UseMutationResult } from "@tanstack/react-query";
import type { Compatibility, CompatibilityEntry, MigrationAction, ScanInput, ScanResponse } from "../../api/migration";
import { CheckboxRow } from "../ui/CheckboxRow";
import { GhostButton } from "../ui/GhostButton";
import { Panel } from "../ui/Panel";
import { PrimaryButton } from "../ui/PrimaryButton";

type ScanMutation = UseMutationResult<ScanResponse, Error, ScanInput>;

interface ModsStepProps {
  scan: ScanMutation;
  decisions: Record<number, MigrationAction>;
  onDecide: (modId: number, action: MigrationAction) => void;
  disableUnavailable: boolean;
  onDisableUnavailable: (value: boolean) => void;
  onBack: () => void;
  onRetry: () => void;
  onContinue: () => void;
}

function nameOf(entry: CompatibilityEntry): string {
  return entry.item.name ?? entry.item.filename;
}

function reasonOf(compatibility: Compatibility): string {
  return compatibility.status === "upgradable" ? "" : compatibility.reason;
}

export function ModsStep({
  scan,
  decisions,
  onDecide,
  disableUnavailable,
  onDisableUnavailable,
  onBack,
  onRetry,
  onContinue,
}: ModsStepProps) {
  if (scan.isPending) {
    return (
      <Panel title="Checking mods">
        <p className="font-mono text-xs tracking-wider text-ink-3 uppercase">Checking every mod and plugin…</p>
      </Panel>
    );
  }
  if (scan.isError) {
    return (
      <Panel title="Checking mods">
        <div className="space-y-4">
          <p className="text-sm text-rose">{scan.error.message}</p>
          <div className="flex flex-wrap justify-between gap-2">
            <GhostButton onPress={onBack}>Back</GhostButton>
            <PrimaryButton onPress={onRetry}>Try again</PrimaryButton>
          </div>
        </div>
      </Panel>
    );
  }
  if (!scan.data) return null;

  const { entries, target } = scan.data.report;
  const version = target.mc_version;
  const groups = {
    upgradable: entries.filter((entry) => entry.compatibility.status === "upgradable"),
    unavailable: entries.filter((entry) => entry.compatibility.status === "unavailable"),
    declared: entries.filter((entry) => entry.compatibility.status === "declared_compatible"),
    undecided: entries.filter((entry) => entry.compatibility.status === "needs_decision"),
  };
  const pending = groups.undecided.filter((entry) => decisions[entry.item.id] === undefined).length;
  const allUpgradable = entries.length > 0 && groups.upgradable.length === entries.length;

  return (
    <div className="space-y-5">
      <Panel title="Mods and plugins">
        <div className="space-y-5">
          {entries.length === 0 ? (
            <p className="text-sm text-ink-2">This server has no mods or plugins to check. Only the server software changes.</p>
          ) : allUpgradable ? (
            <p className="rounded-control border border-green/30 bg-green/10 px-4 py-3 text-sm text-green">
              All {entries.length} mods and plugins can be upgraded to {version}.
            </p>
          ) : (
            <p className="text-sm text-ink-2">
              {groups.upgradable.length} upgradable · {groups.unavailable.length} can't be upgraded · {groups.declared.length}{" "}
              declare support · {groups.undecided.length} need a decision
            </p>
          )}

          {groups.undecided.length > 0 && (
            <section className="space-y-3">
              <header className="space-y-1">
                <h3 className="text-sm font-semibold text-ink-0">Needs your decision ({groups.undecided.length})</h3>
                <p className="text-xs text-ink-3">
                  These jars do not declare support for {version}, or declare no Minecraft version at all. Choose whether
                  each one stays enabled.
                </p>
              </header>
              <div className="flex flex-wrap gap-2">
                <GhostButton onPress={() => groups.undecided.forEach((entry) => onDecide(entry.item.id, "keep"))}>
                  Keep all enabled
                </GhostButton>
                <GhostButton onPress={() => groups.undecided.forEach((entry) => onDecide(entry.item.id, "disable"))}>
                  Disable all
                </GhostButton>
              </div>
              <ul className="space-y-2">
                {groups.undecided.map((entry) => {
                  const choice = decisions[entry.item.id];
                  return (
                    <li key={entry.item.id} className="space-y-2 rounded-control bg-input/60 p-3">
                      <div className="min-w-0">
                        <p className="truncate text-sm text-ink-1">{nameOf(entry)}</p>
                        <p className="text-xs text-ink-3">{reasonOf(entry.compatibility)}</p>
                      </div>
                      <div className="flex flex-wrap gap-2">
                        <Button
                          size="sm"
                          variant={choice === "keep" ? "primary" : "ghost"}
                          aria-pressed={choice === "keep"}
                          onPress={() => onDecide(entry.item.id, "keep")}
                        >
                          Keep enabled
                        </Button>
                        <Button
                          size="sm"
                          variant={choice === "disable" ? "primary" : "ghost"}
                          aria-pressed={choice === "disable"}
                          onPress={() => onDecide(entry.item.id, "disable")}
                        >
                          Disable
                        </Button>
                      </div>
                    </li>
                  );
                })}
              </ul>
            </section>
          )}

          {groups.unavailable.length > 0 && (
            <section className="space-y-3">
              <h3 className="text-sm font-semibold text-ink-0">Can't be upgraded ({groups.unavailable.length})</h3>
              <ul className="space-y-2">
                {groups.unavailable.map((entry) => (
                  <li key={entry.item.id} className="rounded-control bg-input/60 p-3">
                    <p className="truncate text-sm text-ink-1">{nameOf(entry)}</p>
                    <p className="text-xs text-ink-3">{reasonOf(entry.compatibility)}</p>
                  </li>
                ))}
              </ul>
              <CheckboxRow isSelected={disableUnavailable} onChange={onDisableUnavailable}>
                Disable the {groups.unavailable.length} mods that can't be upgraded. They stay on disk and can be re-enabled later.
              </CheckboxRow>
            </section>
          )}

          {groups.upgradable.length > 0 && !allUpgradable && (
            <details className="rounded-control bg-input/40 p-3">
              <summary className="cursor-pointer text-sm text-ink-2">Upgradable ({groups.upgradable.length})</summary>
              <ul className="mt-3 space-y-2">
                {groups.upgradable.map((entry) => (
                  <li key={entry.item.id} className="flex items-center justify-between gap-3 text-sm">
                    <span className="min-w-0 truncate text-ink-1">{nameOf(entry)}</span>
                    <span className="shrink-0 font-mono text-xs text-ink-3">
                      {entry.compatibility.status === "upgradable" ? entry.compatibility.version.version_number : ""}
                    </span>
                  </li>
                ))}
              </ul>
            </details>
          )}

          {groups.declared.length > 0 && (
            <section className="space-y-2">
              <h3 className="text-sm font-semibold text-ink-0">Declares support ({groups.declared.length})</h3>
              <ul className="space-y-1">
                {groups.declared.map((entry) => (
                  <li key={entry.item.id} className="truncate text-xs text-ink-3">
                    {nameOf(entry)} · {reasonOf(entry.compatibility)}
                  </li>
                ))}
              </ul>
            </section>
          )}
        </div>
      </Panel>

      <div className="flex flex-wrap justify-between gap-2">
        <GhostButton onPress={onBack}>Back</GhostButton>
        <PrimaryButton isDisabled={pending > 0} onPress={onContinue}>
          {pending > 0 ? `Decide ${pending} more` : "Review changes"}
        </PrimaryButton>
      </div>
    </div>
  );
}
