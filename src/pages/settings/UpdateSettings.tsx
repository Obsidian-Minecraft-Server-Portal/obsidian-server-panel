import { Icon } from "@iconify-icon/react";
import { useEffect, useState } from "react";
import { ConfirmDialog } from "../../components/files/ConfirmDialog";
import { Chip } from "../../components/ui/Chip";
import { GhostButton } from "../../components/ui/GhostButton";
import { Panel } from "../../components/ui/Panel";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { useApplyUpdate, useCheckUpdates, useUpdateStatus, waitForPanel } from "../../api/updates";
import { Feedback } from "../account/Feedback";
import { errorMessage } from "../account/errorMessage";

const RESTART_TIMEOUT_MESSAGE = "The panel did not come back within 60 seconds. Refresh the page to check the version.";

export function UpdateSettings() {
  const status = useUpdateStatus();
  const check = useCheckUpdates();
  const apply = useApplyUpdate();
  const [confirming, setConfirming] = useState(false);
  const [restartFrom, setRestartFrom] = useState<string | null>(null);
  const [restartTimedOut, setRestartTimedOut] = useState(false);

  const info = check.data ?? status.data;
  const restarting = restartFrom !== null && !restartTimedOut;
  const failure = check.error ?? status.error;

  useEffect(() => {
    if (restartFrom === null) return;
    let active = true;
    waitForPanel(restartFrom).then((back) => {
      if (!active) return;
      if (back) window.location.reload();
      else setRestartTimedOut(true);
    });
    return () => {
      active = false;
    };
  }, [restartFrom]);

  const install = async () => {
    if (!info) return;
    const result = await apply.mutateAsync();
    if (result.restarting) setRestartFrom(info.current_version);
    else check.mutate();
  };

  return (
    <>
      <Panel
        title="Updates"
        actions={
          <GhostButton size="sm" isPending={check.isPending} onPress={() => check.mutate()}>
            <Icon icon="lucide:refresh-cw" width={14} height={14} />
            Check now
          </GhostButton>
        }
      >
        {info && (
          <div className="mb-4">
            {info.update_available ? (
              <Chip tone="accent" pulse>
                Update available
              </Chip>
            ) : (
              <Chip tone="green">Up to date</Chip>
            )}
          </div>
        )}
        <dl className="grid grid-cols-[8rem_1fr] gap-x-4 gap-y-3 text-sm">
          <Row label="Current" value={info?.current_version ?? "…"} />
          <Row label="Latest" value={info ? (info.latest_version ?? "None published") : "…"} />
          <Row label="Checked" value={info ? new Date(info.checked_at).toLocaleString() : "…"} />
        </dl>
        <p className="mt-4 text-xs leading-relaxed text-ink-3">
          The new release is downloaded and the panel restarts in place. Running Minecraft servers keep running.
        </p>
        <div className="mt-5 flex flex-col gap-4">
          <Feedback
            error={restartTimedOut ? RESTART_TIMEOUT_MESSAGE : failure ? errorMessage(failure) : null}
            saved={restarting ? "Update installed. Restarting the panel…" : null}
          />
          {info?.update_available && (
            <div>
              <PrimaryButton isDisabled={restarting || apply.isPending} onPress={() => setConfirming(true)}>
                Install update
              </PrimaryButton>
            </div>
          )}
        </div>
      </Panel>
      {confirming && info && (
        <ConfirmDialog
          title={`Install ${info.latest_version ?? "update"}?`}
          description="The panel restarts to finish the update. Running Minecraft servers keep running because they live in detached runners."
          confirmLabel="Install update"
          onConfirm={install}
          onClose={() => setConfirming(false)}
        />
      )}
    </>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <>
      <dt className="self-center font-mono text-[11px] tracking-wider text-ink-3 uppercase">{label}</dt>
      <dd className="min-w-0 truncate font-mono text-ink-1" title={value}>
        {value}
      </dd>
    </>
  );
}
