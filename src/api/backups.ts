import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import { useTopic } from "./ws";

export interface SnapshotInfo {
  id: string;
  label: string;
  file_count: number;
  size_bytes: number;
  created_at: string;
}

export interface TreeEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size_bytes: number;
}

export interface StoreStats {
  snapshots: number;
  total_bytes: number;
  stored_bytes: number;
  dedup_ratio: number;
}

export interface BackupPolicy {
  keep_hourly: number;
  keep_daily: number;
  keep_weekly: number;
  keep_monthly: number;
  excludes: string[];
}

export interface BackupStarted {
  task_id: number;
}

const backupsKey = (instanceId: number) => ["instances", instanceId, "backups"] as const;
const policyKey = (instanceId: number) => ["instances", instanceId, "backup-policy"] as const;

export function useSnapshots(instanceId: number, enabled = true) {
  return useQuery({
    queryKey: [...backupsKey(instanceId), "list"],
    queryFn: () => api.get<SnapshotInfo[]>(`/instances/${instanceId}/backups`),
    enabled,
  });
}

export function useBackupStats(instanceId: number) {
  return useQuery({
    queryKey: [...backupsKey(instanceId), "stats"],
    queryFn: () => api.get<StoreStats>(`/instances/${instanceId}/backups/stats`),
  });
}

export function useSnapshotTree(instanceId: number, snapshot: string, path: string) {
  return useQuery({
    queryKey: [...backupsKey(instanceId), "tree", snapshot, path],
    queryFn: () =>
      api.get<TreeEntry[]>(
        `/instances/${instanceId}/backups/${encodeURIComponent(snapshot)}/tree?path=${encodeURIComponent(path)}`,
      ),
  });
}

export function useBackupPolicy(instanceId: number) {
  return useQuery({
    queryKey: policyKey(instanceId),
    queryFn: () => api.get<BackupPolicy>(`/instances/${instanceId}/backup-policy`),
  });
}

export function useSaveBackupPolicy(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (policy: BackupPolicy) => api.put<BackupPolicy>(`/instances/${instanceId}/backup-policy`, policy),
    onSuccess: (saved) => queryClient.setQueryData(policyKey(instanceId), saved),
  });
}

export function useCreateBackup(instanceId: number) {
  return useMutation({
    mutationFn: () => api.post<BackupStarted>(`/instances/${instanceId}/backups`),
  });
}

export function useRestoreBackup(instanceId: number) {
  return useMutation({
    mutationFn: ({ snapshot, path }: { snapshot: string; path?: string }) =>
      api.post<BackupStarted>(
        `/instances/${instanceId}/backups/${encodeURIComponent(snapshot)}/restore`,
        path === undefined ? undefined : { path },
      ),
  });
}

export function useDeleteBackup(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (snapshot: string) => api.delete(`/instances/${instanceId}/backups/${encodeURIComponent(snapshot)}`),
    onSettled: () => queryClient.invalidateQueries({ queryKey: backupsKey(instanceId) }),
  });
}

export function snapshotDownloadUrl(instanceId: number, snapshot: string): string {
  return `/api/v1/instances/${instanceId}/backups/${encodeURIComponent(snapshot)}/download`;
}

export function useBackupEvents(instanceId: number) {
  const queryClient = useQueryClient();
  useTopic(`instance.${instanceId}.backups`, () => {
    void queryClient.invalidateQueries({ queryKey: backupsKey(instanceId) });
  });
}
