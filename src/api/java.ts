import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";

export type JavaSource = "piston" | "system";

export interface JavaRuntime {
  id: number;
  source: JavaSource;
  component: string | null;
  version: string;
  path: string;
  arch: string;
  installed_at: string;
}

export interface TaskStarted {
  task_id: number;
}

export const JAVA_COMPONENTS = [
  "java-runtime-alpha",
  "java-runtime-beta",
  "java-runtime-gamma",
  "java-runtime-delta",
  "java-runtime-gamma-snapshot",
  "java-runtime-epsilon",
  "jre-legacy",
] as const;

export const javaKey = ["java"] as const;

export function useJavaRuntimes() {
  return useQuery({
    queryKey: javaKey,
    queryFn: () => api.get<JavaRuntime[]>("/java"),
  });
}

export function useInstallJava() {
  return useMutation({
    mutationFn: (component: string) => api.post<TaskStarted>("/java/install", { component }),
  });
}

export function useDetectJava() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.post<JavaRuntime[]>("/java/detect"),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: javaKey }),
  });
}

export function useRemoveJava() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => api.delete(`/java/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: javaKey }),
  });
}

export interface AvailableJava {
  component: string;
  version: string;
  installed: boolean;
}

export function useAvailableJava() {
  return useQuery({
    queryKey: [...javaKey, "available"],
    queryFn: () => api.get<AvailableJava[]>("/java/available"),
    staleTime: 60 * 60 * 1000,
  });
}

export interface JavaRequirement {
  component: string;
  major_version: number;
}

export function useJavaRequirement(mcVersion: string | null) {
  return useQuery({
    queryKey: [...javaKey, "requirement", mcVersion],
    queryFn: () => api.get<JavaRequirement>(`/java/requirement?mc_version=${encodeURIComponent(mcVersion ?? "")}`),
    enabled: mcVersion !== null && mcVersion !== "",
    staleTime: Infinity,
  });
}
