import { Button } from "@heroui/react";
import { formatDistanceToNow } from "date-fns";
import { useState } from "react";
import { useCreateInvite, useInvites, useRevokeInvite, type Invite } from "../../api/users";
import { Chip } from "../../components/ui/Chip";
import { EmptyState } from "../../components/ui/EmptyState";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { Panel } from "../../components/ui/Panel";
import { CopyButton } from "../account/CopyButton";
import { errorMessage } from "../account/errorMessage";

type InviteState = "used" | "expired" | "active";

function inviteState(invite: Invite): InviteState {
  if (invite.used_at) return "used";
  return Date.parse(invite.expires_at) < Date.now() ? "expired" : "active";
}

const STATE_CHIP: Record<InviteState, { label: string; tone: "cyan" | "rose" | "accent" }> = {
  used: { label: "Used", tone: "cyan" },
  expired: { label: "Expired", tone: "rose" },
  active: { label: "Open", tone: "accent" },
};

export function InvitesPanel() {
  const invites = useInvites();
  const create = useCreateInvite();
  const revoke = useRevokeInvite();
  const [links, setLinks] = useState<Record<number, string>>({});
  const [error, setError] = useState<string | null>(null);

  const issue = () => {
    setError(null);
    create.mutate(undefined, {
      onSuccess: ({ invite, url }) => setLinks((current) => ({ ...current, [invite.id]: url })),
      onError: (err) => setError(errorMessage(err)),
    });
  };

  return (
    <Panel
      title="Invite links"
      actions={
        <PrimaryButton size="sm" onPress={issue} isPending={create.isPending}>
          New invite
        </PrimaryButton>
      }
    >
      <p className="mb-4 text-xs text-ink-3">
        Links work once and expire after three days. A link can only be copied right after it is created.
      </p>
      {error && <p role="alert" className="mb-3 text-sm text-rose">{error}</p>}
      {invites.isPending && <div className="h-16 animate-shimmer rounded-control bg-raised" />}
      {invites.data?.length === 0 && (
        <EmptyState icon="lucide:link" title="No invites yet" description="Create a link and send it to someone you trust." className="py-10" />
      )}
      <ul className="divide-y divide-line">
        {invites.data?.map((invite) => {
          const state = inviteState(invite);
          const chip = STATE_CHIP[state];
          const url = links[invite.id];
          return (
            <li key={invite.id} className="space-y-2 py-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <Chip tone={chip.tone}>{chip.label}</Chip>
                <span className="font-mono text-[11px] text-ink-3 uppercase">
                  {state === "used"
                    ? `used ${formatDistanceToNow(new Date(invite.used_at ?? invite.created_at), { addSuffix: true })}`
                    : `expires ${formatDistanceToNow(new Date(invite.expires_at), { addSuffix: true })}`}
                </span>
              </div>
              {state === "active" && (
                <div className="flex flex-wrap items-center justify-between gap-2">
                  {url ? <CopyButton value={url} label="Copy link" /> : <span className="text-xs text-ink-4">Link was shown when created</span>}
                  <Button
                    size="sm"
                    variant="ghost"
                    isPending={revoke.isPending && revoke.variables === invite.id}
                    onPress={() => revoke.mutate(invite.id)}
                    className="rounded-control text-rose"
                  >
                    Revoke
                  </Button>
                </div>
              )}
            </li>
          );
        })}
      </ul>
    </Panel>
  );
}
