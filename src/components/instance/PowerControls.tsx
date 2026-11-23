import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useEffect, useState, type ComponentProps } from "react";
import { hasPermission, usePowerAction, type InstanceView, type PowerAction } from "../../api/instances";
import { isActive, isTransitioning } from "./status";

const KILL_ARM_MS = 4000;

const TONE: Record<PowerAction, string> = {
  start: "border-accent bg-accent text-accent-ink hover:bg-accent-hover",
  restart: "border-amber/40 bg-amber/10 text-amber hover:bg-amber/20",
  stop: "border-rose/40 bg-rose/10 text-rose hover:bg-rose/20",
  kill: "border-rose bg-rose text-white hover:bg-rose/90",
};

interface PowerControlsProps {
  view: InstanceView;
  size?: "sm" | "md";
}

type PowerButtonProps = ComponentProps<typeof Button> & { action: PowerAction };

function PowerButton({ action, className, ...props }: PowerButtonProps) {
  return (
    <Button
      variant="ghost"
      className={cn(
        "rounded-control border font-semibold transition-colors duration-150 active:scale-[0.98]",
        TONE[action],
        className,
      )}
      {...props}
    />
  );
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
            <PowerButton action="stop" size={size} isDisabled={busy} onPress={() => act("stop")}>
              <Icon icon="lucide:square" width={iconSize} height={iconSize} />
              {status.state === "stopping" ? "Stopping…" : "Stop"}
            </PowerButton>
          )}
          {canRestart && (
            <PowerButton action="restart" size={size} isDisabled={busy} onPress={() => act("restart")}>
              <Icon icon="lucide:rotate-cw" width={iconSize} height={iconSize} />
              Restart
            </PowerButton>
          )}
          {canKill && (
            <PowerButton action="kill" size={size} onPress={() => (killArmed ? act("kill") : setKillArmed(true))}>
              <Icon icon="lucide:zap-off" width={iconSize} height={iconSize} />
              {killArmed ? "Confirm kill" : "Kill"}
            </PowerButton>
          )}
        </>
      ) : (
        canStart && (
          <PowerButton
            action="start"
            size={size}
            isDisabled={busy}
            isPending={power.isPending}
            onPress={() => act("start")}
          >
            <Icon icon="lucide:play" width={iconSize} height={iconSize} />
            Start
          </PowerButton>
        )
      )}
      {power.error && <span className="text-xs text-rose">{power.error.message}</span>}
    </div>
  );
}
