import { UserAvatar } from "../../components/ui/UserAvatar";
import { Button } from "@heroui/react";
import { format } from "date-fns";
import type { UserProfile } from "../../api/auth";
import { useUsers } from "../../api/users";
import { Chip } from "../../components/ui/Chip";
import { EmptyState } from "../../components/ui/EmptyState";

interface UsersTableProps {
  onEdit: (user: UserProfile) => void;
}

export function UsersTable({ onEdit }: UsersTableProps) {
  const users = useUsers();

  if (users.isPending) return <div className="h-40 animate-shimmer rounded-control bg-raised" />;
  if (users.isError) return <p className="text-sm text-rose">{users.error.message}</p>;
  if (users.data.length === 0) {
    return <EmptyState icon="lucide:users" title="No accounts yet" description="Create a user or send an invite link." />;
  }

  return (
    <ul className="divide-y divide-line">
      {users.data.map((user) => (
        <li key={user.id} className="flex flex-wrap items-center gap-4 py-3">
          <UserAvatar name={user.username} />
          <div className="min-w-0 flex-1 space-y-1">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-medium text-ink-0">{user.username}</span>
              {user.is_admin && <Chip tone="accent">Admin</Chip>}
              {user.totp_enabled && <Chip tone="cyan">2FA</Chip>}
            </div>
            <p className="truncate font-mono text-[11px] text-ink-3 uppercase">
              {user.email ?? "no email"} · {user.instance_limit} instance limit · joined{" "}
              {format(new Date(user.created_at), "d MMM yyyy")}
            </p>
          </div>
          <Button size="sm" variant="ghost" onPress={() => onEdit(user)} className="rounded-control border border-line-strong">
            Edit
          </Button>
        </li>
      ))}
    </ul>
  );
}
