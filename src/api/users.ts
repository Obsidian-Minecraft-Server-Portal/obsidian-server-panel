import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import type { UserProfile } from "./auth";
import { currentUserKey } from "./session";

export interface NewUserInput {
  username: string;
  email?: string;
  password: string;
  is_admin: boolean;
  instance_limit?: number;
}

export interface UserUpdateInput {
  email?: string;
  is_admin?: boolean;
  instance_limit?: number;
}

export interface Invite {
  id: number;
  created_by: number | null;
  created_at: string;
  expires_at: string;
  used_at: string | null;
}

export interface CreatedInvite {
  invite: Invite;
  token: string;
  url: string;
}

export interface AcceptInviteInput {
  username: string;
  email?: string;
  password: string;
}

const usersKey = ["users"] as const;
const invitesKey = ["invites"] as const;

export function useUsers() {
  return useQuery({ queryKey: usersKey, queryFn: () => api.get<UserProfile[]>("/users") });
}

export function useCreateUser() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: NewUserInput) => api.post<UserProfile>("/users", body),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: usersKey }),
  });
}

export function useUpdateUser() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: number; body: UserUpdateInput }) =>
      api.patch<UserProfile>(`/users/${id}`, body),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: usersKey });
      void queryClient.invalidateQueries({ queryKey: currentUserKey });
    },
  });
}

export function useDeleteUser() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => api.delete(`/users/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: usersKey }),
  });
}

export function useInvites() {
  return useQuery({ queryKey: invitesKey, queryFn: () => api.get<Invite[]>("/invites") });
}

export function useCreateInvite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.post<CreatedInvite>("/invites", {}),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: invitesKey }),
  });
}

export function useRevokeInvite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => api.delete(`/invites/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: invitesKey }),
  });
}

export function useInviteStatus(token: string) {
  return useQuery({
    queryKey: ["invites", "status", token],
    queryFn: () => api.get<{ valid: boolean }>(`/invites/${encodeURIComponent(token)}`),
    retry: false,
  });
}

export function useAcceptInvite(token: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: AcceptInviteInput) =>
      api.post<UserProfile>(`/invites/${encodeURIComponent(token)}/accept`, body),
    onSuccess: (user) => queryClient.setQueryData(currentUserKey, user),
  });
}

export function useSetUserPassword() {
  return useMutation({
    mutationFn: ({ id, password }: { id: number; password: string }) =>
      api.post<void>(`/users/${id}/password`, { password }),
  });
}
