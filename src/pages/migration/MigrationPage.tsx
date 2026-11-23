import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router";
import { hasPermission, useInstance, usePowerAction } from "../../api/instances";
import { useMigrationScan, useStartMigration, type MigrationAction, type StartedMigration } from "../../api/migration";
import { isActive } from "../../components/instance/status";
import { useInstanceId } from "../../components/instance/useInstanceId";
import { ModsStep } from "../../components/migration/ModsStep";
import { ReviewStep } from "../../components/migration/ReviewStep";
import { RunStep } from "../../components/migration/RunStep";
import { TargetStep } from "../../components/migration/TargetStep";
import { LOADER_META } from "../../components/runtime/loaderMeta";
import { GhostButton } from "../../components/ui/GhostButton";
import { Panel } from "../../components/ui/Panel";

const STEPS = ["Target", "Mods", "Review", "Run"];

export function MigrationPage() {
  const instanceId = useInstanceId();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const instance = useInstance(instanceId);
  const power = usePowerAction(instanceId);
  const scan = useMigrationScan(instanceId);
  const start = useStartMigration(instanceId);
  const [step, setStep] = useState(0);
  const [mcVersion, setMcVersion] = useState<string | null>(params.get("mc_version"));
  const [loaderVersion, setLoaderVersion] = useState<string | null>(null);
  const [decisions, setDecisions] = useState<Record<number, MigrationAction>>({});
  const [disableUnavailable, setDisableUnavailable] = useState(true);
  const [backup, setBackup] = useState(true);
  const [acknowledged, setAcknowledged] = useState(false);
  const [started, setStarted] = useState<StartedMigration | null>(null);

  const view = instance.data;
  const openInstance = () => navigate(`/instances/${instanceId}/settings`);

  if (instance.isError) {
    return <p className="px-4 py-5 text-sm text-rose md:px-8 md:py-6">{instance.error.message}</p>;
  }
  if (!view) return <p className="px-4 py-5 text-sm text-ink-3 md:px-8 md:py-6">Loading the server…</p>;

  const blocked = (message: string) => (
    <div className="space-y-4 px-4 py-5 md:px-8 md:py-6">
      <p className="text-sm text-ink-2">{message}</p>
      <GhostButton onPress={openInstance}>Back to instance</GhostButton>
    </div>
  );
  if (view.loader === "custom") {
    return blocked("A custom server jar is uploaded, not installed, so it cannot be migrated automatically.");
  }
  const canMigrate =
    hasPermission(view, "settings.edit") && hasPermission(view, "content.install") && hasPermission(view, "content.remove");
  if (!canMigrate) {
    return blocked("You need settings, content install and content remove permissions to migrate this server.");
  }
  const running = isActive(view.status) || view.status.installing;
  const canBackup = hasPermission(view, "backups.create");

  const continueFromTarget = () => {
    if (!mcVersion || running) return;
    scan.mutate({ mc_version: mcVersion, loader_version: loaderVersion });
    setDecisions({});
    setStep(1);
  };

  const startMigration = () => {
    if (!mcVersion || running) return;
    start.mutate(
      {
        mc_version: mcVersion,
        loader_version: loaderVersion,
        backup: backup && canBackup,
        disable_unavailable: disableUnavailable,
        decisions: Object.entries(decisions).map(([modId, action]) => ({ mod_id: Number(modId), action })),
      },
      {
        onSuccess: (result) => {
          setStarted(result);
          setStep(3);
        },
      },
    );
  };

  return (
    <div className="space-y-6 px-4 py-5 md:px-8 md:py-6">
      <header className="flex flex-wrap items-center gap-4">
        <GhostButton isIconOnly aria-label="Back to instance" onPress={openInstance} className="size-10">
          <Icon icon="lucide:chevron-left" width={18} height={18} />
        </GhostButton>
        <div className="min-w-0 flex-1 space-y-1">
          <h1 className="truncate text-2xl font-semibold tracking-tight text-ink-0">Migrate server</h1>
          <p className="truncate font-mono text-[11px] text-ink-3">
            {view.name} · {LOADER_META[view.loader].label} {view.mc_version}
          </p>
        </div>
        <ol aria-label="Steps" className="flex w-full items-center gap-4 font-mono text-[11px] tracking-wider text-ink-3 uppercase md:w-auto">
          {STEPS.map((label, index) => (
            <li
              key={label}
              aria-current={index === step ? "step" : undefined}
              className={index === step ? "text-accent" : index < step ? "text-ink-2" : undefined}
            >
              {index + 1}. {label}
            </li>
          ))}
        </ol>
      </header>

      {running && step < 3 && (
        <Panel>
          <p className="text-sm text-amber">
            Stop the server before migrating it. <Link to={`/instances/${instanceId}/console`} className="text-accent hover:underline">Open the console</Link>
            {hasPermission(view, "power.stop") && (
              <>
                {" "}
                <GhostButton isDisabled={power.isPending} onPress={() => power.mutate("stop")}>
                  Stop server
                </GhostButton>
              </>
            )}
          </p>
        </Panel>
      )}

      {step === 0 && (
        <TargetStep
          view={view}
          mcVersion={mcVersion}
          loaderVersion={loaderVersion}
          onMcVersion={setMcVersion}
          onLoaderVersion={setLoaderVersion}
          onBack={openInstance}
          onContinue={continueFromTarget}
        />
      )}
      {step === 1 && (
        <ModsStep
          scan={scan}
          decisions={decisions}
          onDecide={(modId, action) => setDecisions((current) => ({ ...current, [modId]: action }))}
          disableUnavailable={disableUnavailable}
          onDisableUnavailable={setDisableUnavailable}
          onBack={() => setStep(0)}
          onRetry={continueFromTarget}
          onContinue={() => setStep(2)}
        />
      )}
      {step === 2 && scan.data && (
        <ReviewStep
          scan={scan.data}
          decisions={decisions}
          disableUnavailable={disableUnavailable}
          backup={backup}
          onBackup={setBackup}
          canBackup={canBackup}
          acknowledged={acknowledged}
          onAcknowledge={setAcknowledged}
          isPending={start.isPending}
          blocked={running}
          error={start.error?.message ?? null}
          onBack={() => setStep(1)}
          onStart={startMigration}
        />
      )}
      {step === 3 && started && (
        <RunStep
          instanceId={instanceId}
          started={started}
          canStart={hasPermission(view, "power.start")}
          onOpenInstance={openInstance}
        />
      )}
    </div>
  );
}
