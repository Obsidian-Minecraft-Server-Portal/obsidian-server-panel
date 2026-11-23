import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import { REFRESH_MS } from "./polling";

export type Weekday = "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun";

export type Trigger =
  | { kind: "interval"; every: number; unit: "minutes" | "hours" }
  | { kind: "daily"; time: string }
  | { kind: "weekly"; days: Weekday[]; time: string }
  | { kind: "monthly"; day: number; time: string }
  | { kind: "once"; at: string }
  | { kind: "on_start" }
  | { kind: "on_stop" }
  | { kind: "on_crash" }
  | { kind: "empty_for"; minutes: number }
  | { kind: "player_joined" };

export type Action =
  | { type: "backup" }
  | { type: "restart"; countdown_secs: number[] }
  | { type: "command"; line: string }
  | { type: "start" }
  | { type: "stop" }
  | { type: "update_content" };

export interface Step {
  action: Action;
  delay_secs: number;
  only_if_running: boolean;
}

export interface Schedule {
  name: string;
  enabled: boolean;
  trigger: Trigger;
  steps: Step[];
}

export interface ScheduleRecord {
  id: number;
  instance_id: number;
  description: string;
  schedule: Schedule;
  next_run_at: string | null;
  created_at: string;
  updated_at: string;
}

export type RunStatus = "running" | "succeeded" | "failed";

export interface ScheduleRun {
  id: number;
  schedule_id: number;
  started_at: string;
  finished_at: string | null;
  status: RunStatus;
  message: string;
}

export interface SchedulePreview {
  description: string;
  next_runs: string[];
}

const schedulesKey = (instanceId: number) => ["instances", instanceId, "schedules"] as const;

export function useSchedules(instanceId: number) {
  return useQuery({
    queryKey: schedulesKey(instanceId),
    queryFn: () => api.get<ScheduleRecord[]>(`/instances/${instanceId}/schedules`),
    refetchInterval: REFRESH_MS.list,
  });
}

export function useScheduleRuns(scheduleId: number) {
  return useQuery({
    queryKey: ["schedules", scheduleId, "runs"],
    queryFn: () => api.get<ScheduleRun[]>(`/schedules/${scheduleId}/runs`),
    refetchInterval: (query) => (query.state.data?.some((run) => run.status === "running") ? 2_000 : false),
  });
}

export function usePreviewSchedule(schedule: Schedule | null) {
  return useQuery({
    queryKey: ["schedules", "preview", schedule],
    queryFn: () => api.post<SchedulePreview>("/schedules/preview", { schedule }),
    enabled: schedule !== null,
    placeholderData: keepPreviousData,
    retry: false,
  });
}

function useInvalidateSchedules(instanceId: number) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: schedulesKey(instanceId) });
}

export function useCreateSchedule(instanceId: number) {
  const invalidate = useInvalidateSchedules(instanceId);
  return useMutation({
    mutationFn: (schedule: Schedule) => api.post<ScheduleRecord>(`/instances/${instanceId}/schedules`, schedule),
    onSettled: invalidate,
  });
}

export function useUpdateSchedule(instanceId: number) {
  const queryClient = useQueryClient();
  const invalidate = useInvalidateSchedules(instanceId);
  return useMutation({
    mutationFn: ({ id, patch }: { id: number; patch: Partial<Schedule> }) =>
      api.patch<ScheduleRecord>(`/schedules/${id}`, patch),
    onMutate: () => queryClient.cancelQueries({ queryKey: schedulesKey(instanceId) }),
    onSettled: invalidate,
  });
}

export function useDeleteSchedule(instanceId: number) {
  const invalidate = useInvalidateSchedules(instanceId);
  return useMutation({
    mutationFn: (id: number) => api.delete(`/schedules/${id}`),
    onSettled: invalidate,
  });
}

export function useRunSchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => api.post(`/schedules/${id}/run`),
    onSettled: (_, __, id) => queryClient.invalidateQueries({ queryKey: ["schedules", id, "runs"] }),
  });
}
