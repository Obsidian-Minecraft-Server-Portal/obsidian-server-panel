import { Icon } from "@iconify-icon/react";
import { useNavigate } from "react-router";
import { useLogout } from "../../api/auth";
import { useCurrentUser } from "../../api/session";
import { EmptyState } from "../../components/ui/EmptyState";
import { GhostButton } from "../../components/ui/GhostButton";
import { PageHeader } from "../../components/ui/PageHeader";
import { Panel } from "../../components/ui/Panel";
import { ApiTokensPanel } from "./ApiTokensPanel";
import { EmailPanel } from "./EmailPanel";
import { PasswordPanel } from "./PasswordPanel";
import { TotpPanel } from "./TotpPanel";
import { SecurityEventsPanel } from "./SecurityEventsPanel";
import { TrustedIpsPanel } from "./TrustedIpsPanel";

export function AccountPage() {
  const me = useCurrentUser();
  const logout = useLogout();
  const navigate = useNavigate();
  const user = me.data;

  const signOut = () =>
    logout.mutate(undefined, { onSettled: () => navigate("/login", { replace: true }) });

  return (
    <>
      <PageHeader
        title="Account"
        subtitle={user ? (user.email ? `${user.username} · ${user.email}` : user.username) : "Loading your account"}
        actions={
          <GhostButton onPress={signOut} isPending={logout.isPending}>
            <Icon icon="lucide:log-out" width={16} height={16} />
            Sign out
          </GhostButton>
        }
      />
      {me.isError && (
        <div className="px-8 pb-8">
          <EmptyState icon="lucide:triangle-alert" title="Could not load your account" description={me.error.message} />
        </div>
      )}
      {user && (
        <div className="grid gap-6 px-8 pb-8 xl:grid-cols-2">
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
          <Panel title="API tokens" className="xl:col-span-2">
            <ApiTokensPanel />
          </Panel>
          <Panel title="Recent activity" className="xl:col-span-2">
            <SecurityEventsPanel />
          </Panel>
        </div>
      )}
    </>
  );
}
