import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";

export type SmtpTlsMode = "none" | "starttls" | "tls";

export interface SmtpView {
  host: string;
  port: number;
  username: string | null;
  from: string;
  tls: SmtpTlsMode;
  password_set: boolean;
}

export interface SettingsView {
  timezone: string;
  upnp_default: boolean;
  upnp_default_source: "env" | "panel" | null;
  stop_timeout_secs: number;
  stop_timeout_source: "env" | "panel" | null;
  public_url: string | null;
  public_url_source: "config" | "panel" | null;
  smtp: SmtpView | null;
  smtp_source: "config" | "panel" | null;
}

export interface SmtpInput {
  host: string;
  port: number;
  username?: string;
  password?: string;
  from: string;
  tls: SmtpTlsMode;
}

export interface SettingsUpdateInput {
  timezone?: string;
  upnp_default?: boolean;
  stop_timeout_secs?: number;
  public_url?: string;
  smtp?: SmtpInput;
}

export const settingsKey = ["settings"] as const;

export function useSettings() {
  return useQuery({ queryKey: settingsKey, queryFn: () => api.get<SettingsView>("/settings") });
}

export function useUpdateSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: SettingsUpdateInput) => api.put<SettingsView>("/settings", body),
    onSuccess: (view) => queryClient.setQueryData(settingsKey, view),
  });
}

export function useRemoveSmtp() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.delete("/settings/smtp"),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: settingsKey }),
  });
}

export function useTestSmtp() {
  return useMutation({ mutationFn: () => api.post<void>("/settings/smtp/test") });
}
