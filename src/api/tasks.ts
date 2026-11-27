import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { api } from "./client";
import { useTopic, useWebSocket } from "./ws";

export type TaskStatus = "running" | "finished" | "failed";

export interface TaskView {
  id: number;
  kind: string;
  title: string;
  owner: number | null;
  status: TaskStatus;
  progress: number;
  message: string;
  result: Record<string, unknown> | null;
  error: string | null;
  created_at: string;
  updated_at: string;
}

interface TaskRemoval {
  id: number;
  removed: true;
}

export const tasksKey = ["tasks"] as const;

function applyTaskEvent(list: TaskView[], event: TaskView | TaskRemoval): TaskView[] {
  if ("removed" in event) return list.filter((task) => task.id !== event.id);
  return [...list.filter((task) => task.id !== event.id), event];
}

export function useTasks() {
  const queryClient = useQueryClient();
  const { status } = useWebSocket();
  const lastStatus = useRef(status);
  useEffect(() => {
    if (status === "open" && lastStatus.current !== "open") {
      void queryClient.invalidateQueries({ queryKey: tasksKey, refetchType: "active" }, { cancelRefetch: false });
    }
    lastStatus.current = status;
  }, [status, queryClient]);
  useTopic("tasks", (payload) => {
    queryClient.setQueryData<TaskView[]>(tasksKey, (list) =>
      list && applyTaskEvent(list, payload as TaskView | TaskRemoval),
    );
  });
  return useQuery({
    queryKey: tasksKey,
    queryFn: () => api.get<TaskView[]>("/tasks"),
  });
}

export function useTaskById(taskId: number | null): TaskView | undefined {
  const { data } = useTasks();
  return taskId === null ? undefined : data?.find((entry) => entry.id === taskId);
}

export function useDismissTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => api.delete(`/tasks/${id}`),
    onSuccess: (_, id) =>
      queryClient.setQueryData<TaskView[]>(tasksKey, (list) => list?.filter((task) => task.id !== id)),
  });
}
