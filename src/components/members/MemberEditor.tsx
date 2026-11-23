import { Modal } from "@heroui/react";
import { useEffect, useState } from "react";
import { useUpdateMember, type MemberView, type PermissionCatalog } from "../../api/members";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { PermissionMatrix } from "./PermissionMatrix";
import { RoleSegment } from "./RoleSegment";
import { sameMembers } from "./format";

interface MemberEditorProps {
  instanceId: number;
  member: MemberView;
  catalog: PermissionCatalog;
  canEdit: boolean;
  onClose: () => void;
}

export function MemberEditor({ instanceId, member, catalog, canEdit, onClose }: MemberEditorProps) {
  const update = useUpdateMember(instanceId);
  const [draft, setDraft] = useState(member.permissions);

  useEffect(() => {
    setDraft(member.permissions);
  }, [member.permissions]);

  const dirty = !sameMembers(draft, member.permissions);
  const roles = catalog.roles.map((preset) => preset.role);
  const busy = update.isPending;

  return (
    <Modal.Backdrop isOpen onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="lg">
        <Modal.Dialog className="bg-panel text-ink-1">
          <Modal.Header className="flex flex-wrap items-center justify-between gap-3">
            <Modal.Heading className="text-lg font-semibold text-ink-0">Permissions · {member.username}</Modal.Heading>
            {canEdit && (
              <RoleSegment
                roles={roles}
                value={member.role}
                disabled={busy}
                onChange={(role) => update.mutate({ userId: member.user_id, role })}
              />
            )}
          </Modal.Header>
          <Modal.Body className="space-y-4">
            <PermissionMatrix catalog={catalog.permissions} value={draft} onChange={setDraft} disabled={!canEdit || busy} />
            {update.isError && <p className="text-sm text-rose">{update.error.message}</p>}
          </Modal.Body>
          <Modal.Footer className="flex justify-end gap-2">
            <GhostButton onPress={onClose}>Close</GhostButton>
            <PrimaryButton
              isDisabled={!canEdit || !dirty || busy}
              onPress={() => update.mutate({ userId: member.user_id, permissions: draft })}
            >
              Save permissions
            </PrimaryButton>
          </Modal.Footer>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}
