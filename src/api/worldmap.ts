import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { MapManifest, MapSource } from "../components/worldmap/map-source";
import { api, BASE } from "./client";
import { isInstanceId } from "./instances";

export interface WorldMapSettings {
  enabled: boolean;
  consent: boolean;
  watchWhileRunning: boolean;
  renderAfterStop: boolean;
  fullPassHours: number;
}

export interface WorldMapDimension {
  id: string;
  folder: string;
  regions: number;
}

export interface WorldMapStatus {
  enabled: boolean;
  rendering: boolean;
  lastError: string | null;
  packKey: string | null;
  lastFullPass: number | null;
  regions: number;
  dimensions: WorldMapDimension[];
}

export interface WorldMapTile {
  x: number;
  z: number;
  file: string;
}

export interface WorldMapManifest extends MapManifest {
  packKey: string;
  tiles: WorldMapTile[];
}

export const worldMapKey = (instanceId: number) => ["instances", instanceId, "worldmap"] as const;

export function useWorldMapSettings(instanceId: number) {
  return useQuery({
    queryKey: [...worldMapKey(instanceId), "settings"] as const,
    queryFn: () => api.get<WorldMapSettings>(`/instances/${instanceId}/worldmap/settings`),
    enabled: isInstanceId(instanceId),
  });
}

export function useUpdateWorldMapSettings(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: WorldMapSettings) => api.put<WorldMapSettings>(`/instances/${instanceId}/worldmap/settings`, settings),
    onSuccess: (saved) => {
      queryClient.setQueryData([...worldMapKey(instanceId), "settings"], saved);
      void queryClient.invalidateQueries({ queryKey: [...worldMapKey(instanceId), "status"] });
    },
  });
}

export function useWorldMapStatus(instanceId: number, enabled = true) {
  return useQuery({
    queryKey: [...worldMapKey(instanceId), "status"] as const,
    queryFn: () => api.get<WorldMapStatus>(`/instances/${instanceId}/worldmap/status`),
    enabled: enabled && isInstanceId(instanceId),
    refetchInterval: (query) => (query.state.data?.rendering ? 4000 : 30000),
  });
}

export function useRenderWorldMap(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.post(`/instances/${instanceId}/worldmap/render`),
    onSettled: () => void queryClient.invalidateQueries({ queryKey: [...worldMapKey(instanceId), "status"] }),
  });
}

export function useWorldMapManifest(instanceId: number, folder: string) {
  return useQuery({
    queryKey: [...worldMapKey(instanceId), "manifest", folder] as const,
    queryFn: () => api.get<WorldMapManifest>(`/instances/${instanceId}/worldmap/dimensions/${folder}`),
    enabled: isInstanceId(instanceId),
    staleTime: 0,
  });
}

/** The viewer's source for one dimension. Files are served from the instance's map folder. */
export function worldMapSource(instanceId: number, folder: string): MapSource {
  const files = `${BASE}/instances/${instanceId}/worldmap/files/`;
  return {
    loadManifest: () => api.get<WorldMapManifest>(`/instances/${instanceId}/worldmap/dimensions/${folder}`),
    fileUrl: (file) => `${files}${file}`,
  };
}

export function dimensionLabel(id: string): string {
  switch (id) {
    case "minecraft:overworld":
      return "Overworld";
    case "minecraft:the_nether":
      return "Nether";
    case "minecraft:the_end":
      return "End";
    default:
      return id.replace(/^minecraft:/, "").replace(/_/g, " ");
  }
}

export function renderSummary(status: WorldMapStatus): string {
  if (status.rendering) return "Rendering…";
  if (status.lastError) return "Last render failed";
  if (status.lastFullPass) return `Full render ${new Date(status.lastFullPass * 1000).toLocaleString()}`;
  return "Waiting for the first full render";
}
