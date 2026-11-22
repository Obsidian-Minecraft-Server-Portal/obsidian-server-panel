import { useQuery } from "@tanstack/react-query";
import { api } from "./client";
import type { CurrentUser, Health } from "./types";

export const healthKey = ["health"] as const;
export const currentUserKey = ["auth", "me"] as const;

export function useHealth() {
  return useQuery({
    queryKey: healthKey,
    queryFn: () => api.get<Health>("/health"),
    retry: false,
  });
}

export function useCurrentUser(enabled = true) {
  return useQuery({
    queryKey: currentUserKey,
    queryFn: () => api.get<CurrentUser>("/auth/me"),
    enabled,
    retry: false,
  });
}
