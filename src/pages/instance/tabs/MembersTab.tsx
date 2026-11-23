import { useState } from "react";
import { useNavigate } from "react-router";
import { useCurrentUser } from "../../../api/session";
import { useMembers, usePermissionCatalog, useRemoveMember } from "../../../api/members";
import { InviteMemberModal } from "../../../components/members/InviteMemberModal";
import { MemberEditor } from "../../../components/members/MemberEditor";
import { MemberRow } from "../../../components/members/MemberRow";
import { Chip } from "../../../components/ui/Chip";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Panel } from "../../../components/ui/Panel";
import { PrimaryButton } from "../../../components/ui/PrimaryButton";

export function MembersSection({ instanceId }: { instanceId: number }) {
  const navigate = useNavigate();
  const me = useCurrentUser();
  const catalog = usePermissionCatalog();
  const members = useMembers(instanceId);
  const remove = useRemoveMember(instanceId);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [inviting, setInviting] = useState(false);

  const failure = members.error ?? catalog.error;
  if (failure) {
    return (
      <div className="px-8 pb-8">
        <EmptyState icon="lucide:shield-alert" title="Members unavailable" description={failure.message} />
      </div>
    );
  }
  if (!members.data || !catalog.data) {
    return (
      <div className="px-8 pb-8">
        <p className="font-mono text-xs text-ink-3">LOADING MEMBERS…</p>
      </div>
    );
  }

  const list = members.data;
  const myId = me.data?.id;
  const mine = list.find((member) => member.user_id === myId);
  const canManage =
    me.data?.is_admin === true || mine?.is_owner === true || mine?.permissions.includes("members.manage") === true;
  const selected = list.find((member) => member.user_id === selectedId) ?? null;

  const removeMember = (userId: number) => {
    if (userId === selectedId) setSelectedId(null);
    remove.mutate(userId, {
      onSuccess: () => {
        if (userId === myId && !me.data?.is_admin) navigate("/");
      },
    });
  };

  return (
    <div>
      <Panel
        title="Members"
        actions={
          <div className="flex flex-wrap items-center gap-2">
            <Chip>
              {list.length} {list.length === 1 ? "member" : "members"}
            </Chip>
            {canManage && <PrimaryButton onPress={() => setInviting(true)}>Invite member</PrimaryButton>}
          </div>
        }
      >
        <ul className="space-y-2">
          {list.map((member) => (
            <MemberRow
              key={member.user_id}
              member={member}
              active={member.user_id === selectedId}
              canEdit={canManage && !member.is_owner}
              canRemove={!member.is_owner && (canManage || member.user_id === myId)}
              onEdit={() => setSelectedId(member.user_id)}
              onRemove={() => removeMember(member.user_id)}
            />
          ))}
        </ul>
        {remove.isError && <p className="mt-3 text-sm text-rose">{remove.error.message}</p>}
      </Panel>

      {selected && (
        <MemberEditor
          key={selected.user_id}
          instanceId={instanceId}
          member={selected}
          catalog={catalog.data}
          canEdit={canManage}
          onClose={() => setSelectedId(null)}
        />
      )}
      {inviting && <InviteMemberModal instanceId={instanceId} catalog={catalog.data} onClose={() => setInviting(false)} />}
    </div>
  );
}
