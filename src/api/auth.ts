import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import { currentUserKey, healthKey } from "./session";
import type { SmtpInput } from "./settings";
import type { CurrentUser, Health } from "./types";

export interface UserProfile extends CurrentUser {
  instance_limit: number;
  created_at: string;
}

export interface LoginBody {
  username: string;
  password: string;
  totp_code?: string;
  email_code?: string;
}

export interface LoginResponse {
  status: "authenticated" | "totp_required" | "email_code_required";
  user?: UserProfile;
}

export interface SetupBody {
  username: string;
  email?: string;
  password: string;
  timezone?: string;
  upnp_default?: boolean;
  public_url?: string;
  smtp?: SmtpInput;
}

export interface TotpSetup {
  secret: string;
  otpauth_url: string;
  qr_svg: string;
}

export interface RecoveryCodes {
  recovery_codes: string[];
}

export interface ApiTokenInfo {
  id: number;
  name: string;
  created_at: string;
  expires_at: string | null;
  last_used_at: string | null;
}

export interface NewApiToken extends ApiTokenInfo {
  token: string;
}

export interface TrustedIp {
  id: number;
  ip: string;
  user_agent: string | null;
  first_seen: string;
  last_seen: string;
}

const tokensKey = ["account", "tokens"] as const;
const trustedIpsKey = ["account", "trusted-ips"] as const;

export function useLogin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: LoginBody) => api.post<LoginResponse>("/auth/login", body),
    onSuccess: ({ user }) => {
      if (user) queryClient.setQueryData(currentUserKey, user);
    },
  });
}

export function useLogout() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.post<void>("/auth/logout"),
    onSettled: () => queryClient.removeQueries({ queryKey: currentUserKey }),
  });
}

export function useCompleteSetup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: SetupBody) => api.post<UserProfile>("/setup", body),
    onSuccess: (user) => {
      queryClient.setQueryData<Health>(healthKey, (health) => health && { ...health, setup_required: false });
      queryClient.setQueryData(currentUserKey, user);
    },
  });
}

export function useForgotPassword() {
  return useMutation({
    mutationFn: (email: string) => api.post<void>("/auth/forgot-password", { email }),
  });
}

export function useResetPassword() {
  return useMutation({
    mutationFn: (body: { token: string; password: string }) => api.post<void>("/auth/reset-password", body),
  });
}

export function useUpdateAccount() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: { password: string; email: string }) => api.patch<UserProfile>("/account", body),
    onSuccess: (user) => queryClient.setQueryData(currentUserKey, user),
  });
}

export function useChangePassword() {
  return useMutation({
    mutationFn: (body: { current_password: string; new_password: string }) =>
      api.post<void>("/account/password", body),
  });
}

export function useTotpSetup() {
  return useMutation({ mutationFn: () => api.post<TotpSetup>("/account/totp/setup") });
}

export function useTotpConfirm() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (code: string) => api.post<RecoveryCodes>("/account/totp/confirm", { code }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: currentUserKey }),
  });
}

export function useTotpDisable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (password: string) => api.post<void>("/account/totp/disable", { password }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: currentUserKey }),
  });
}

export function useRegenerateRecoveryCodes() {
  return useMutation({
    mutationFn: (code: string) => api.post<RecoveryCodes>("/account/totp/recovery-codes", { code }),
  });
}

export function useApiTokens() {
  return useQuery({ queryKey: tokensKey, queryFn: () => api.get<ApiTokenInfo[]>("/account/tokens") });
}

export function useCreateApiToken() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: { name: string; expires_in_days: number | null }) =>
      api.post<NewApiToken>("/account/tokens", body),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: tokensKey }),
  });
}

export function useRevokeApiToken() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => api.delete(`/account/tokens/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: tokensKey }),
  });
}

export function useTrustedIps() {
  return useQuery({ queryKey: trustedIpsKey, queryFn: () => api.get<TrustedIp[]>("/account/trusted-ips") });
}

export function useRevokeTrustedIp() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => api.delete(`/account/trusted-ips/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: trustedIpsKey }),
  });
}

export interface SecurityEvent {
  id: number;
  kind: string;
  ip: string | null;
  details: string | null;
  created_at: string;
}

export function useSecurityEvents() {
  return useQuery({
    queryKey: ["account", "security-events"],
    queryFn: () => api.get<SecurityEvent[]>("/account/security-events"),
  });
}
