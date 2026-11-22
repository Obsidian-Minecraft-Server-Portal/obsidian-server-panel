import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";

export interface MemberView {
  user_id: number;
  username: string;
  role: string;
  is_owner: boolean;
  permissions: string[];
}

export interface RolePreset {
  role: string;
  permissions: string[];
}

export interface PermissionCatalog {
  permissions: string[];
  roles: RolePreset[];
}

export interface AddMemberInput {
  username: string;
  role: string;
}

export interface UpdateMemberInput {
  userId: number;
  role?: string;
  permissions?: string[];
}

const membersKey = (instanceId: number) => ["instances", instanceId, "members"] as const;

export function usePermissionCatalog() {
  return useQuery({
    queryKey: ["permissions"],
    queryFn: () => api.get<PermissionCatalog>("/permissions"),
    staleTime: Infinity,
  });
}

export function useMembers(instanceId: number) {
  return useQuery({
    queryKey: membersKey(instanceId),
    queryFn: () => api.get<MemberView[]>(`/instances/${instanceId}/members`),
  });
}

export function useAddMember(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: AddMemberInput) => api.post<MemberView>(`/instances/${instanceId}/members`, input),
    onSettled: () => queryClient.invalidateQueries({ queryKey: membersKey(instanceId) }),
  });
}

export function useUpdateMember(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ userId, ...body }: UpdateMemberInput) =>
      api.patch<MemberView>(`/instances/${instanceId}/members/${userId}`, body),
    onSettled: () => queryClient.invalidateQueries({ queryKey: membersKey(instanceId) }),
  });
}

export function useRemoveMember(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (userId: number) => api.delete(`/instances/${instanceId}/members/${userId}`),
    onSettled: () => queryClient.invalidateQueries({ queryKey: membersKey(instanceId) }),
  });
}
