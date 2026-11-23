import { Icon } from "@iconify-icon/react";
import { useNavigate } from "react-router";
import { useLogout } from "../../api/auth";
import { useCurrentUser } from "../../api/session";
import { EmptyState } from "../../components/ui/EmptyState";
import { GhostButton } from "../../components/ui/GhostButton";
import { Panel } from "../../components/ui/Panel";
import { ApiTokensPanel } from "../account/ApiTokensPanel";
import { EmailPanel } from "../account/EmailPanel";
import { PasswordPanel } from "../account/PasswordPanel";
import { SecurityEventsPanel } from "../account/SecurityEventsPanel";
import { TotpPanel } from "../account/TotpPanel";
import { TrustedIpsPanel } from "../account/TrustedIpsPanel";

export function AccountSettings() {
  const me = useCurrentUser();
  const logout = useLogout();
  const navigate = useNavigate();

  if (me.isPending) return <div className="h-64 animate-shimmer rounded-panel bg-raised" />;
  if (me.isError) {
    return <EmptyState icon="lucide:triangle-alert" title="Could not load your account" description={me.error.message} />;
  }

  const user = me.data;
  const signOut = () => logout.mutate(undefined, { onSettled: () => navigate("/login", { replace: true }) });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <div className="min-w-0 space-y-1">
          <h2 className="text-xl font-semibold text-ink-0">Account</h2>
          <p className="truncate text-sm text-ink-3">{user.email ? `${user.username} · ${user.email}` : user.username}</p>
        </div>
        <GhostButton onPress={signOut} isPending={logout.isPending}>
          <Icon icon="lucide:log-out" width={16} height={16} />
          Sign out
        </GhostButton>
      </div>
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <Panel title="Email address">
          <EmailPanel user={user} />
        </Panel>
        <Panel title="Password">
          <PasswordPanel />
        </Panel>
        <Panel title="Two-factor authentication">
          <TotpPanel enabled={user.totp_enabled} />
        </Panel>
        <Panel title="Trusted addresses">
          <TrustedIpsPanel />
        </Panel>
        <Panel title="API tokens" className="md:col-span-2">
          <ApiTokensPanel />
        </Panel>
        <Panel title="Recent activity" className="md:col-span-2">
          <SecurityEventsPanel />
        </Panel>
      </div>
    </div>
  );
}
