import { formatDistanceToNow } from "date-fns";
import { useSecurityEvents } from "../../api/auth";
import { EmptyState } from "../../components/ui/EmptyState";

const LABELS: Record<string, string> = {
  login_success: "Signed in",
  login_failed: "Failed sign-in",
  new_ip_login: "Signed in from a new address",
  email_code: "Email sign-in code sent",
  totp: "Two-factor code used",
  totp_enabled: "Two-factor turned on",
  totp_disabled: "Two-factor turned off",
  password_changed: "Password changed",
  password_reset: "Password reset by email",
  password_reset_by_admin: "Password replaced by an administrator",
  api_token_created: "API token created",
  api_token_revoked: "API token revoked",
  trusted_ip_revoked: "Trusted address removed",
  invite_accepted: "Account created from invite",
  invite_created: "Invite created",
  invite_revoked: "Invite revoked",
  user_created: "Account created",
  user_updated: "Account updated",
  user_deleted: "Account deleted",
};

export function SecurityEventsPanel() {
  const events = useSecurityEvents();

  if (events.isPending) return <div className="h-16 animate-shimmer rounded-control bg-raised" />;
  if (events.isError) return <p className="text-sm text-rose">{events.error.message}</p>;
  if (events.data.length === 0) {
    return <EmptyState icon="lucide:history" title="No activity yet" description="Sign-ins and security changes appear here." className="py-10" />;
  }

  return (
    <ul className="max-h-96 divide-y divide-line overflow-y-auto">
      {events.data.map((event) => (
        <li key={event.id} className="flex items-center justify-between gap-4 py-3">
          <div className="min-w-0 space-y-0.5">
            <p className="text-sm text-ink-1">{LABELS[event.kind] ?? event.kind.replace(/_/g, " ")}</p>
            {event.details && <p className="truncate text-xs text-ink-3">{event.details}</p>}
          </div>
          <p className="shrink-0 text-right font-mono text-[11px] text-ink-3 uppercase">
            {event.ip && <span className="block">{event.ip}</span>}
            {formatDistanceToNow(new Date(event.created_at), { addSuffix: true })}
          </p>
        </li>
      ))}
    </ul>
  );
}
