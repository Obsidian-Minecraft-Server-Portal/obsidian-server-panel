import { Icon } from "@iconify-icon/react";
import { useEffect, useState } from "react";
import { hasPermission, usePowerAction, type InstanceView, type PowerAction } from "../../api/instances";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { isActive, isTransitioning } from "./status";

const KILL_ARM_MS = 4000;

interface PowerControlsProps {
  view: InstanceView;
  size?: "sm" | "md";
}

export function PowerControls({ view, size = "md" }: PowerControlsProps) {
  const { id, status } = view;
  const power = usePowerAction(id);
  const [killArmed, setKillArmed] = useState(false);
  const iconSize = size === "sm" ? 14 : 16;
  const busy = power.isPending || isTransitioning(status);

  useEffect(() => {
    if (!killArmed) return;
    const timer = setTimeout(() => setKillArmed(false), KILL_ARM_MS);
    return () => clearTimeout(timer);
  }, [killArmed]);

  const act = (action: PowerAction) => {
    setKillArmed(false);
    power.mutate(action);
  };

  if (status.installing) {
    return <span className="font-mono text-xs text-ink-3">Installing…</span>;
  }

  const canStart = hasPermission(view, "power.start");
  const canStop = hasPermission(view, "power.stop");
  const canRestart = hasPermission(view, "power.restart");
  const canKill = hasPermission(view, "power.kill");

  return (
    <div className="flex flex-wrap items-center gap-2">
      {isActive(status) ? (
        <>
          {canStop && (
            <GhostButton size={size} isDisabled={busy} onPress={() => act("stop")}>
              <Icon icon="lucide:square" width={iconSize} height={iconSize} />
              {status.state === "stopping" ? "Stopping…" : "Stop"}
            </GhostButton>
          )}
          {canRestart && (
            <GhostButton size={size} isDisabled={busy} onPress={() => act("restart")}>
              <Icon icon="lucide:rotate-cw" width={iconSize} height={iconSize} />
              Restart
            </GhostButton>
          )}
          {canKill && (
            <GhostButton
              size={size}
              onPress={() => (killArmed ? act("kill") : setKillArmed(true))}
              className="text-rose"
            >
              <Icon icon="lucide:zap-off" width={iconSize} height={iconSize} />
              {killArmed ? "Confirm kill" : "Kill"}
            </GhostButton>
          )}
        </>
      ) : (
        canStart && (
          <PrimaryButton size={size} isDisabled={busy} isPending={power.isPending} onPress={() => act("start")}>
            <Icon icon="lucide:play" width={iconSize} height={iconSize} />
            Start
          </PrimaryButton>
        )
      )}
      {power.error && <span className="text-xs text-rose">{power.error.message}</span>}
    </div>
  );
}
