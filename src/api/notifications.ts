import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import type { Notification } from "./types";
import { useTopic } from "./ws";

export const notificationsKey = ["notifications"] as const;

export function useNotifications() {
  const queryClient = useQueryClient();
  useTopic("notifications", () => void queryClient.invalidateQueries({ queryKey: notificationsKey }));
  return useQuery({
    queryKey: notificationsKey,
    queryFn: () => api.get<Notification[]>("/notifications"),
  });
}

export function useMarkNotificationsRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.post<void>("/notifications/read", {}),
    onSuccess: () =>
      queryClient.setQueryData<Notification[]>(notificationsKey, (list) =>
        list?.map((item) => ({ ...item, read: true })),
      ),
  });
}
