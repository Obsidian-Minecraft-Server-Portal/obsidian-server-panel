import { useQuery } from "@tanstack/react-query";
import { api } from "./client";

export interface MemoryLimits {
  total_mb: number;
  usable_mb: number;
  recommended_mb: number;
}

export function useMemoryLimits() {
  return useQuery({
    queryKey: ["system", "memory"],
    queryFn: () => api.get<MemoryLimits>("/system/memory"),
    staleTime: Infinity,
  });
}

export function useNextPort(enabled: boolean) {
  return useQuery({
    queryKey: ["instances", "next-port"],
    queryFn: () => api.get<{ port: number }>("/instances/next-port"),
    enabled,
    staleTime: 0,
  });
}
