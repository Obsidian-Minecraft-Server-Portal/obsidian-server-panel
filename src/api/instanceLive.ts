import { useQueryClient, type QueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import {
  consoleKey,
  instanceStatsKey,
  patchInstanceStatus,
  type InstanceStats,
  type InstanceStatus,
} from "./instances";
import { useTopic, useWebSocket } from "./ws";

export const MAX_LIVE_CONSOLE_LINES = 1000;

export function statusTopic(id: number) {
  return `instance.${id}.status`;
}

export function statsTopic(id: number) {
  return `instance.${id}.stats`;
}

export function consoleTopic(id: number) {
  return `instance.${id}.console`;
}

export function useLiveInstances(ids: readonly number[], { stats = false }: { stats?: boolean } = {}) {
  const queryClient = useQueryClient();
  const { subscribe } = useWebSocket();
  const idKey = ids.join(",");

  useEffect(() => {
    const unsubscribers = idKey
      .split(",")
      .filter(Boolean)
      .flatMap((raw) => {
        const id = Number(raw);
        const subscriptions = [
          subscribe(statusTopic(id), (payload) => patchInstanceStatus(queryClient, id, payload as InstanceStatus)),
        ];
        if (stats) {
          subscriptions.push(
            subscribe(statsTopic(id), (payload) =>
              queryClient.setQueryData<InstanceStats>(instanceStatsKey(id), payload as InstanceStats),
            ),
          );
        }
        return subscriptions;
      });
    return () => unsubscribers.forEach((unsubscribe) => unsubscribe());
  }, [idKey, stats, subscribe, queryClient]);
}

export function useLiveConsole(id: number) {
  const queryClient = useQueryClient();
  const { status } = useWebSocket();
  const wasOpen = useRef(false);

  useTopic(consoleTopic(id), (payload) => {
    const { line } = payload as { line: string };
    queryClient.setQueryData<string[]>(consoleKey(id), (lines) => lines && appendLine(lines, line));
  });

  useEffect(() => {
    if (status === "open" && !wasOpen.current) {
      refetchAfterReconnect(queryClient, id);
    }
    wasOpen.current = status === "open";
  }, [status, id, queryClient]);
}

function appendLine(lines: string[], line: string): string[] {
  const next = lines.length >= MAX_LIVE_CONSOLE_LINES ? lines.slice(lines.length - MAX_LIVE_CONSOLE_LINES + 1) : lines.slice();
  next.push(line);
  return next;
}

function refetchAfterReconnect(queryClient: QueryClient, id: number) {
  if (queryClient.getQueryState(consoleKey(id))?.dataUpdateCount) {
    void queryClient.invalidateQueries({ queryKey: consoleKey(id), exact: true });
  }
}
