import { useQueryClient } from "@tanstack/react-query";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { CommandInput } from "../../../components/instance/console/CommandInput";
import { ConsoleView } from "../../../components/instance/console/ConsoleView";
import { isActive, isRunning } from "../../../components/instance/status";
import { StatusChip } from "../../../components/instance/InstanceChips";
import { StatsStrip } from "../../../components/instance/stats/StatsStrip";
import { useStatsHistory } from "../../../components/instance/stats/useStatsHistory";
import { consoleKey, useConsoleBacklog, useInstance, useSendCommand } from "../../../api/instances";
import { useLiveConsole } from "../../../api/instanceLive";
import { GhostButton } from "../../../components/ui/GhostButton";
import { PlayersPanels } from "./PlayersTab";

export function ConsoleTab() {
  const id = useInstanceId();
  const queryClient = useQueryClient();
  const instance = useInstance(id);
  const backlog = useConsoleBacklog(id);
  const send = useSendCommand(id);
  useLiveConsole(id);

  const lines = backlog.data ?? [];
  const status = instance.data?.status;
  const active = status ? isActive(status) : false;
  const running = status ? isRunning(status) : false;
  const history = useStatsHistory(id, running);

  return (
    <div className="grid items-stretch gap-5 px-8 pb-8 xl:h-[calc(100dvh-17.5rem)] xl:min-h-[30rem] xl:grid-cols-[20rem_minmax(0,1fr)]">
      <aside className="flex min-h-0 min-w-0 flex-col">
        <PlayersPanels />
      </aside>
      <div className="flex min-h-0 min-w-0 flex-col gap-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            {instance.data && <StatusChip instance={instance.data} />}
            <span className="font-mono text-xs text-ink-3">{lines.length} lines</span>
          </div>
          <div className="flex items-center gap-2">
            {send.error && <span className="text-xs text-rose">{send.error.message}</span>}
            <GhostButton size="sm" onPress={() => queryClient.setQueryData(consoleKey(id), [])}>
              Clear
            </GhostButton>
          </div>
        </div>
        <StatsStrip
          running={running}
          stats={history.stats}
          cpu={history.cpu}
          memory={history.memory}
          memoryMaxMb={instance.data?.memory_max_mb ?? 0}
        />
        {backlog.isError && <p className="text-sm text-rose">{backlog.error.message}</p>}
        <ConsoleView lines={lines} />
        <CommandInput disabled={!active} onSubmit={(line) => send.mutate(line)} />
      </div>
    </div>
  );
}
