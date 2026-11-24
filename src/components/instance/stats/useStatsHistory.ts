import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useLiveInstances } from "../../../api/instanceLive";
import { instanceStatsKey, useInstanceStats, type InstanceStats } from "../../../api/instances";

export const HISTORY_SAMPLES = 90;

export interface StatsHistory {
  stats: InstanceStats | undefined;
  cpu: number[];
  memory: number[];
}

interface Series {
  id: number;
  cpu: number[];
  memory: number[];
}

const EMPTY_SERIES: Series = { id: 0, cpu: [], memory: [] };

function append(values: number[], value: number): number[] {
  const next = values.length >= HISTORY_SAMPLES ? values.slice(1) : values.slice();
  next.push(value);
  return next;
}

export function useStatsHistory(id: number, running: boolean): StatsHistory {
  useLiveInstances([id], { status: false, stats: true });
  const queryClient = useQueryClient();
  const { data, dataUpdatedAt } = useInstanceStats(id, running);
  const [series, setSeries] = useState<Series>(EMPTY_SERIES);

  useEffect(() => {
    if (!running) {
      queryClient.removeQueries({ queryKey: instanceStatsKey(id), exact: true });
      setSeries(EMPTY_SERIES);
    }
  }, [id, running, queryClient]);

  useEffect(() => {
    if (!running || !data) return;
    setSeries((prev) => {
      const base = prev.id === id ? prev : EMPTY_SERIES;
      return { id, cpu: append(base.cpu, data.cpu_percent), memory: append(base.memory, data.memory_bytes) };
    });
  }, [id, data, dataUpdatedAt, running]);

  return {
    stats: running ? data : undefined,
    cpu: series.id === id ? series.cpu : [],
    memory: series.id === id ? series.memory : [],
  };
}
