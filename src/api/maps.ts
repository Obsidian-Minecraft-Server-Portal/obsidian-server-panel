import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import { isInstanceId, upnpKey } from "./instances";

export type MapKind = "bluemap" | "dynmap" | "squaremap" | "pl3xmap";

export interface MapSettings {
  enabled: boolean;
  port: number;
  bind: string | null;
}

export interface DetectedMap {
  kind: MapKind;
  label: string;
  config_path: string | null;
  settings: MapSettings;
  editable: boolean;
  error: string | null;
}

export interface MapSettingsUpdate {
  enabled?: boolean;
  port?: number;
  bind?: string;
}

export const mapsKey = (instanceId: number) => ["instances", instanceId, "maps"] as const;

export function mapPath(instanceId: number, kind: MapKind): string {
  return `/api/v1/instances/${instanceId}/map/${kind}/`;
}

export function useMaps(instanceId: number, enabled = true) {
  return useQuery({
    queryKey: mapsKey(instanceId),
    queryFn: () => api.get<DetectedMap[]>(`/instances/${instanceId}/maps`),
    enabled: enabled && isInstanceId(instanceId),
  });
}

export function useUpdateMap(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ kind, update }: { kind: MapKind; update: MapSettingsUpdate }) =>
      api.put<DetectedMap>(`/instances/${instanceId}/maps/${kind}`, update),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: mapsKey(instanceId) }),
  });
}

export function useSetMapUpnp(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (enabled: boolean) => api.put(`/instances/${instanceId}/map-upnp`, { enabled }),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: upnpKey(instanceId) }),
  });
}

export interface ConfigEntry {
  path: string;
  value: string;
  kind: "bool" | "number" | "text";
}

export function useMapEntries(instanceId: number, kind: MapKind, enabled: boolean) {
  return useQuery({
    queryKey: [...mapsKey(instanceId), kind, "entries"] as const,
    queryFn: () => api.get<ConfigEntry[]>(`/instances/${instanceId}/maps/${kind}/entries`),
    enabled: enabled && isInstanceId(instanceId),
  });
}

export function useUpdateMapEntry(instanceId: number, kind: MapKind) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (update: { path: string; value: string }) =>
      api.put<ConfigEntry[]>(`/instances/${instanceId}/maps/${kind}/entries`, update),
    onSuccess: (entries) => {
      queryClient.setQueryData([...mapsKey(instanceId), kind, "entries"], entries);
      void queryClient.invalidateQueries({ queryKey: mapsKey(instanceId) });
    },
  });
}
