import { useQuery } from "@tanstack/react-query";
import { api } from "./client";

export type LoaderKind =
  | "vanilla"
  | "fabric"
  | "quilt"
  | "forge"
  | "neo_forge"
  | "paper"
  | "purpur"
  | "spigot"
  | "craft_bukkit"
  | "custom";

export const loaderKindsKey = ["loaders"] as const;

export function useLoaderKinds() {
  return useQuery({
    queryKey: loaderKindsKey,
    queryFn: () => api.get<LoaderKind[]>("/loaders"),
    staleTime: Infinity,
  });
}

export function useMinecraftVersions(kind: LoaderKind | null) {
  return useQuery({
    queryKey: ["loaders", kind, "versions"],
    queryFn: () => api.get<string[]>(`/loaders/${kind}/versions`),
    enabled: kind !== null,
    staleTime: Infinity,
  });
}

export function useLoaderVersions(kind: LoaderKind | null, mcVersion: string | null) {
  return useQuery({
    queryKey: ["loaders", kind, "versions", mcVersion],
    queryFn: () => api.get<string[]>(`/loaders/${kind}/versions/${encodeURIComponent(mcVersion ?? "")}`),
    enabled: kind !== null && mcVersion !== null,
    staleTime: Infinity,
  });
}
