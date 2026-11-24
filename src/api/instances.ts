import { useMutation, useQuery, useQueryClient, type QueryClient } from "@tanstack/react-query";
import { api, upload } from "./client";
import { REFRESH_MS } from "./polling";
import type { LoaderKind } from "./loaders";

export type RunState = "starting" | "running" | "stopping" | "stopped" | "crashed";

export interface InstanceStatus {
  state: RunState;
  pid: number | null;
  started_at: string | null;
  installing: boolean;
  crash_hints?: string[];
}

export interface Instance {
  id: number;
  owner_id: number;
  name: string;
  loader: LoaderKind;
  mc_version: string;
  loader_version: string | null;
  java_runtime_id: number | null;
  jvm_args: string;
  memory_min_mb: number;
  memory_max_mb: number;
  port: number;
  upnp_enabled: boolean;
  auto_start: boolean;
  icon_path: string | null;
  banner_path: string | null;
  directory: string;
  created_at: string;
  updated_at: string;
}

export type InstancePermission =
  | "console.read"
  | "console.send"
  | "power.start"
  | "power.stop"
  | "power.restart"
  | "power.kill"
  | "files.read"
  | "files.write"
  | "files.delete"
  | "files.sftp"
  | "content.install"
  | "content.remove"
  | "backups.create"
  | "backups.restore"
  | "backups.download"
  | "backups.delete"
  | "schedules.manage"
  | "players.manage"
  | "settings.edit"
  | "members.manage";

export interface InstanceView extends Instance {
  status: InstanceStatus;
  permissions: string[];
}

export function hasPermission(view: Pick<InstanceView, "permissions">, permission: InstancePermission): boolean {
  return view.permissions.includes(permission);
}

export interface InstanceStats {
  cpu_percent: number;
  memory_bytes: number;
  players_online: number | null;
  max_players: number | null;
  motd: string | null;
  tps: number | null;
}

export interface Player {
  name: string;
  uuid: string | null;
  online: boolean;
  op: boolean;
  whitelisted: boolean;
  banned: boolean;
}

export type PlayerAction = "op" | "deop" | "whitelist" | "unwhitelist" | "kick" | "ban" | "pardon";

export type PowerAction = "start" | "stop" | "restart" | "kill";

export interface UpnpInfo {
  enabled: boolean;
  mapped: boolean;
  external_ip: string | null;
  external_port: number | null;
}

export interface CreateInstanceInput {
  name: string;
  loader: LoaderKind;
  mc_version: string;
  loader_version?: string | null;
  java_runtime_id?: number | null;
  java_component?: string | null;
  port?: number | null;
  upnp_enabled?: boolean | null;
  accept_eula: boolean;
}

export type UpdateInstanceInput = Partial<
  Pick<
    Instance,
    | "name"
    | "jvm_args"
    | "memory_min_mb"
    | "memory_max_mb"
    | "java_runtime_id"
    | "port"
    | "upnp_enabled"
    | "auto_start"
    | "loader"
    | "mc_version"
    | "loader_version"
  >
>;

export type Properties = Record<string, string>;

export const instancesKey = ["instances"] as const;
export const instanceKey = (id: number) => ["instances", id] as const;
export const instanceStatsKey = (id: number) => ["instances", id, "stats"] as const;
export const consoleKey = (id: number) => ["instances", id, "console"] as const;
export const playersKey = (id: number) => ["instances", id, "players"] as const;
export const propertiesKey = (id: number) => ["instances", id, "properties"] as const;
export const upnpKey = (id: number) => ["instances", id, "upnp"] as const;

export const CONSOLE_BACKLOG_LINES = 500;
const PLAYER_REFRESH_MS = 15_000;
const UPNP_REFRESH_MS = 15_000;

export function isInstanceId(id: number): boolean {
  return Number.isInteger(id) && id > 0;
}

export function iconUrl(id: number, version: string): string {
  return `/api/v1/instances/${id}/icon?v=${encodeURIComponent(version)}`;
}

export function bannerUrl(id: number, version: string): string {
  return `/api/v1/instances/${id}/banner?v=${encodeURIComponent(version)}`;
}

export function putInstance(queryClient: QueryClient, view: InstanceView) {
  queryClient.setQueryData(instanceKey(view.id), view);
  queryClient.setQueryData<InstanceView[]>(instancesKey, (list) => {
    if (!list) return list;
    return list.some((item) => item.id === view.id)
      ? list.map((item) => (item.id === view.id ? view : item))
      : [...list, view];
  });
}

export function patchInstanceStatus(queryClient: QueryClient, id: number, status: InstanceStatus) {
  queryClient.setQueryData<InstanceView>(instanceKey(id), (view) => view && { ...view, status });
  queryClient.setQueryData<InstanceView[]>(instancesKey, (list) =>
    list?.map((item) => (item.id === id ? { ...item, status } : item)),
  );
}

export function useInstances() {
  return useQuery({
    queryKey: instancesKey,
    queryFn: () => api.get<InstanceView[]>("/instances"),
    refetchInterval: REFRESH_MS.list,
  });
}

export function useInstance(id: number) {
  return useQuery({
    queryKey: instanceKey(id),
    queryFn: () => api.get<InstanceView>(`/instances/${id}`),
    refetchInterval: REFRESH_MS.list,
    enabled: isInstanceId(id),
  });
}

export function instanceStatsOptions(id: number) {
  return {
    queryKey: instanceStatsKey(id),
    queryFn: () => api.get<InstanceStats>(`/instances/${id}/stats`),
    staleTime: Infinity,
  };
}

export function useInstanceStats(id: number, enabled = true) {
  return useQuery({ ...instanceStatsOptions(id), enabled: enabled && isInstanceId(id) });
}

export function useUpdateInstance(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (patch: UpdateInstanceInput) => api.patch<InstanceView>(`/instances/${id}`, patch),
    onSuccess: (view) => putInstance(queryClient, view),
  });
}

export function useDeleteInstance(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.delete(`/instances/${id}`),
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: instanceKey(id) });
      queryClient.setQueryData<InstanceView[]>(instancesKey, (list) => list?.filter((item) => item.id !== id));
    },
  });
}

export function useReinstallInstance(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.post<InstanceView>(`/instances/${id}/install`),
    onSuccess: (view) => putInstance(queryClient, view),
  });
}

export function usePowerAction(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (action: PowerAction) => api.post<InstanceStatus>(`/instances/${id}/power`, { action }),
    onSuccess: (status) => patchInstanceStatus(queryClient, id, status),
  });
}

export function useSendCommand(id: number) {
  return useMutation({
    mutationFn: (line: string) => api.post<void>(`/instances/${id}/command`, { line }),
  });
}

export function useConsoleBacklog(id: number) {
  return useQuery({
    queryKey: consoleKey(id),
    queryFn: () => api.get<string[]>(`/instances/${id}/console?lines=${CONSOLE_BACKLOG_LINES}`),
    staleTime: Infinity,
    enabled: isInstanceId(id),
  });
}

export function usePlayers(id: number) {
  return useQuery({
    queryKey: playersKey(id),
    queryFn: () => api.get<Player[]>(`/instances/${id}/players`),
    refetchInterval: PLAYER_REFRESH_MS,
    enabled: isInstanceId(id),
  });
}

export function usePlayerAction(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name, action }: { name: string; action: PlayerAction }) =>
      api.post<void>(`/instances/${id}/players/${action}`, { name }),
    onSettled: () => queryClient.invalidateQueries({ queryKey: playersKey(id), exact: true }),
  });
}

export function useProperties(id: number) {
  return useQuery({
    queryKey: propertiesKey(id),
    queryFn: () => api.get<Properties>(`/instances/${id}/properties`),
    enabled: isInstanceId(id),
  });
}

export function useSaveProperties(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (properties: Properties) => api.put<void>(`/instances/${id}/properties`, properties),
    onSuccess: (_, properties) => {
      queryClient.setQueryData(propertiesKey(id), properties);
      void queryClient.invalidateQueries({ queryKey: propertiesKey(id), exact: true });
      void queryClient.invalidateQueries({ queryKey: instanceKey(id), exact: true });
    },
  });
}

export function useUpnp(id: number, enabled = true) {
  return useQuery({
    queryKey: upnpKey(id),
    queryFn: () => api.get<UpnpInfo>(`/instances/${id}/upnp`),
    refetchInterval: UPNP_REFRESH_MS,
    enabled: enabled && isInstanceId(id),
  });
}

function invalidateInstanceViews(queryClient: QueryClient, id: number) {
  void queryClient.invalidateQueries({ queryKey: instanceKey(id), exact: true });
  void queryClient.invalidateQueries({ queryKey: instancesKey, exact: true });
}

export function useUploadIcon(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (image: Blob) => {
      const form = new FormData();
      form.append("icon", image, "icon.png");
      return upload<void>(`/instances/${id}/icon`, form, undefined, undefined, "PUT");
    },
    onSuccess: () => invalidateInstanceViews(queryClient, id),
  });
}

export function useUploadBanner(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (image: Blob) => {
      const form = new FormData();
      form.append("banner", image, "banner.png");
      return upload<void>(`/instances/${id}/banner`, form, undefined, undefined, "PUT");
    },
    onSuccess: () => invalidateInstanceViews(queryClient, id),
  });
}

export function useRemoveBanner(id: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.delete(`/instances/${id}/banner`),
    onSuccess: () => invalidateInstanceViews(queryClient, id),
  });
}
