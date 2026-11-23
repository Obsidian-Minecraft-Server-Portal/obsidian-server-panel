import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import type { InstalledContent, PlatformId, ProjectVersion } from "./content";
import type { LoaderKind } from "./loaders";

export interface PlannedDependency {
  platform: PlatformId;
  project_id: string;
  version: ProjectVersion;
}

export type Compatibility =
  | { status: "upgradable"; version: ProjectVersion; dependencies: PlannedDependency[]; note: string | null }
  | { status: "unavailable"; reason: string }
  | { status: "declared_compatible"; reason: string; minecraft_range: string }
  | { status: "needs_decision"; reason: string; minecraft_range: string | null };

export interface CompatibilityEntry {
  item: InstalledContent;
  compatibility: Compatibility;
}

export interface MigrationTarget {
  loader: LoaderKind;
  mc_version: string;
  loader_version: string | null;
}

export interface SoftwareState {
  loader: LoaderKind;
  mc_version: string;
  loader_version: string | null;
  java_runtime_id: number | null;
}

export interface ScanCounts {
  upgradable: number;
  unavailable: number;
  declared_compatible: number;
  needs_decision: number;
}

export interface ScanReport {
  current: SoftwareState;
  target: MigrationTarget;
  counts: ScanCounts;
  entries: CompatibilityEntry[];
}

export interface ScanResponse {
  report: ScanReport;
  java_component: string | null;
  java_major_version: number | null;
}

export interface ScanInput {
  mc_version: string;
  loader_version: string | null;
}

export type MigrationAction = "keep" | "disable";

export interface StartMigrationInput extends ScanInput {
  backup: boolean;
  disable_unavailable: boolean;
  decisions: { mod_id: number; action: MigrationAction }[];
}

export interface StartedMigration {
  migration_id: number;
  task_id: number;
}

export interface RunReport {
  upgraded: string[];
  added: string[];
  disabled: string[];
}

export type MigrationStatus = "running" | "completed" | "failed" | "rolled_back";

export interface MigrationRecord {
  id: number;
  instance_id: number;
  status: MigrationStatus;
  target: MigrationTarget;
  snapshot_id: string | null;
  error: string | null;
  report: RunReport | null;
  can_roll_back: boolean;
  created_at: number;
  finished_at: number | null;
}

export const migrationsKey = (instanceId: number) => ["instances", instanceId, "migrations"] as const;

export function useMigrations(instanceId: number) {
  return useQuery({
    queryKey: migrationsKey(instanceId),
    queryFn: () => api.get<MigrationRecord[]>(`/instances/${instanceId}/migrations`),
  });
}

export function useMigrationScan(instanceId: number) {
  return useMutation({
    mutationFn: (input: ScanInput) => api.post<ScanResponse>(`/instances/${instanceId}/migration/scan`, input),
  });
}

export function useStartMigration(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: StartMigrationInput) => api.post<StartedMigration>(`/instances/${instanceId}/migration`, input),
    onSettled: () => queryClient.invalidateQueries({ queryKey: migrationsKey(instanceId) }),
  });
}

export function useRollbackMigration(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (migrationId: number) =>
      api.post<{ task_id: number }>(`/instances/${instanceId}/migrations/${migrationId}/rollback`),
    onSettled: () => queryClient.invalidateQueries({ queryKey: migrationsKey(instanceId) }),
  });
}
