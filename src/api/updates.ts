import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import type { Health } from "./types";

export interface UpdateStatus {
  current_version: string;
  latest_version: string | null;
  update_available: boolean;
  checked_at: string;
}

export interface ApplyResponse {
  updated: boolean;
  version: string | null;
  restarting: boolean;
}

export const updateStatusKey = ["updates"] as const;

export function useUpdateStatus() {
  return useQuery({
    queryKey: updateStatusKey,
    queryFn: () => api.get<UpdateStatus>("/updates"),
    retry: false,
  });
}

export function useCheckUpdates() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.get<UpdateStatus>("/updates?refresh=true"),
    onSuccess: (status) => queryClient.setQueryData(updateStatusKey, status),
  });
}

export function useApplyUpdate() {
  return useMutation({
    mutationFn: () => api.post<ApplyResponse>("/updates/apply"),
  });
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export async function waitForPanel(previousVersion: string, timeoutMs = 60_000, intervalMs = 2_000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  let unreachable = false;
  while (Date.now() < deadline) {
    await sleep(intervalMs);
    try {
      const health = await api.get<Health>("/health");
      if (health.version !== previousVersion || unreachable) return true;
    } catch {
      unreachable = true;
    }
  }
  return false;
}
