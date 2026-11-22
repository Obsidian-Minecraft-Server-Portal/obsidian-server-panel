import { useState } from "react";
import { useNavigate, useParams } from "react-router";
import { useCurrentUser } from "../../../api/session";
import { useMembers, usePermissionCatalog, useRemoveMember } from "../../../api/members";
import { InviteMemberPanel } from "../../../components/members/InviteMemberPanel";
import { MemberEditor } from "../../../components/members/MemberEditor";
import { MemberRow } from "../../../components/members/MemberRow";
import { Chip } from "../../../components/ui/Chip";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Panel } from "../../../components/ui/Panel";

export function MembersTab() {
  const { id = "" } = useParams();
  const instanceId = Number(id);
  const navigate = useNavigate();
  const me = useCurrentUser();
  const catalog = usePermissionCatalog();
  const members = useMembers(instanceId);
  const remove = useRemoveMember(instanceId);
  const [selectedId, setSelectedId] = useState<number | null>(null);

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
        if (userId === myId && !me.data?.is_admin) navigate("/instances");
      },
    });
  };

  return (
    <div className="space-y-6 px-8 pb-8">
      <Panel title="Members" actions={<Chip>{list.length} {list.length === 1 ? "member" : "members"}</Chip>}>
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
        />
      )}

      {canManage && <InviteMemberPanel instanceId={instanceId} catalog={catalog.data} />}
    </div>
  );
}
