import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, upload } from "./client";
import type { ContentSource, VersionRef } from "./content";

export interface ResourcePack {
  id: number;
  name: string;
  size: number;
  sha1: string;
  source: ContentSource | null;
}

export interface ResourcePackList {
  packs: ResourcePack[];
  url: string | null;
  public_url_configured: boolean;
}

export const resourcePacksKey = (instanceId: number) => ["instances", instanceId, "resource-packs"] as const;

const packsKey = resourcePacksKey;

export function useResourcePacks(instanceId: number, enabled = true) {
  return useQuery({
    queryKey: packsKey(instanceId),
    queryFn: () => api.get<ResourcePackList>(`/instances/${instanceId}/resource-packs`),
    enabled,
  });
}

export function useAddResourcePack(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: VersionRef) =>
      api.post<ResourcePackList>(`/instances/${instanceId}/resource-packs`, {
        platform: input.platform,
        project_id: input.projectId,
        version_id: input.versionId,
      }),
    onSuccess: (list) => queryClient.setQueryData(packsKey(instanceId), list),
  });
}

export function useUploadResourcePack(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (file: File) => {
      const form = new FormData();
      form.append("pack", file, file.name);
      return upload<ResourcePackList>(`/instances/${instanceId}/resource-packs/upload`, form);
    },
    onSuccess: (list) => queryClient.setQueryData(packsKey(instanceId), list),
  });
}

export function useReorderResourcePacks(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (ids: number[]) => api.put<ResourcePackList>(`/instances/${instanceId}/resource-packs/order`, { ids }),
    onMutate: async (ids) => {
      await queryClient.cancelQueries({ queryKey: packsKey(instanceId) });
      const previous = queryClient.getQueryData<ResourcePackList>(packsKey(instanceId));
      if (previous) {
        const byId = new Map(previous.packs.map((pack) => [pack.id, pack]));
        const packs = ids.flatMap((id) => byId.get(id) ?? []);
        queryClient.setQueryData<ResourcePackList>(packsKey(instanceId), { ...previous, packs });
      }
      return { previous };
    },
    onError: (_error, _ids, context) => {
      if (context?.previous) queryClient.setQueryData(packsKey(instanceId), context.previous);
    },
    onSuccess: (list) => queryClient.setQueryData(packsKey(instanceId), list),
  });
}

export function useRemoveResourcePack(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (packId: number) => api.delete<ResourcePackList>(`/instances/${instanceId}/resource-packs/${packId}`),
    onSuccess: (list) => queryClient.setQueryData(packsKey(instanceId), list),
  });
}
