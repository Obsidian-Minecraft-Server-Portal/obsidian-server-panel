import { useEffect, useState } from "react";
import { useUpdateMember, type MemberView, type PermissionCatalog } from "../../api/members";
import { Panel } from "../ui/Panel";
import { PrimaryButton } from "../ui/PrimaryButton";
import { PermissionMatrix } from "./PermissionMatrix";
import { RoleSegment } from "./RoleSegment";
import { sameMembers } from "./format";

interface MemberEditorProps {
  instanceId: number;
  member: MemberView;
  catalog: PermissionCatalog;
  canEdit: boolean;
}

export function MemberEditor({ instanceId, member, catalog, canEdit }: MemberEditorProps) {
  const update = useUpdateMember(instanceId);
  const [draft, setDraft] = useState(member.permissions);

  useEffect(() => {
    setDraft(member.permissions);
  }, [member.permissions]);

  const dirty = !sameMembers(draft, member.permissions);
  const roles = catalog.roles.map((preset) => preset.role);
  const busy = update.isPending;

  return (
    <Panel
      title={`Permissions · ${member.username}`}
      actions={
        canEdit && (
          <RoleSegment
            roles={roles}
            value={member.role}
            disabled={busy}
            onChange={(role) => update.mutate({ userId: member.user_id, role })}
          />
        )
      }
    >
      <div className="space-y-4">
        <PermissionMatrix catalog={catalog.permissions} value={draft} onChange={setDraft} disabled={!canEdit || busy} />
        <div className="flex items-center justify-between gap-3">
          {update.isError ? <p className="text-sm text-rose">{update.error.message}</p> : <span />}
          <PrimaryButton
            isDisabled={!canEdit || !dirty || busy}
            onPress={() => update.mutate({ userId: member.user_id, permissions: draft })}
          >
            Save permissions
          </PrimaryButton>
        </div>
      </div>
    </Panel>
  );
}
