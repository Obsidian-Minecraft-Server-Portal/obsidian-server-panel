import { Input, TextField } from "@heroui/react";
import { useState, type FormEvent } from "react";
import { useAddMember, useUpdateMember, type PermissionCatalog } from "../../api/members";
import { Panel } from "../ui/Panel";
import { PrimaryButton } from "../ui/PrimaryButton";
import { PermissionMatrix } from "./PermissionMatrix";
import { RoleSegment } from "./RoleSegment";
import { sameMembers } from "./format";

interface InviteMemberPanelProps {
  instanceId: number;
  catalog: PermissionCatalog;
}

export function InviteMemberPanel({ instanceId, catalog }: InviteMemberPanelProps) {
  const roles = catalog.roles.map((preset) => preset.role);
  const presetFor = (role: string) => catalog.roles.find((preset) => preset.role === role)?.permissions ?? [];

  const [username, setUsername] = useState("");
  const [role, setRole] = useState(roles[0] ?? "viewer");
  const [draft, setDraft] = useState(() => presetFor(role));
  const add = useAddMember(instanceId);
  const update = useUpdateMember(instanceId);
  const busy = add.isPending || update.isPending;
  const error = add.error ?? update.error;

  const chooseRole = (next: string) => {
    setRole(next);
    setDraft(presetFor(next));
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const name = username.trim();
    if (!name) return;
    update.reset();
    const member = await add.mutateAsync({ username: name, role }).catch(() => null);
    if (!member) return;
    setUsername("");
    if (!sameMembers(draft, member.permissions)) {
      update.mutate({ userId: member.user_id, permissions: draft });
    }
  };

  return (
    <Panel title="Invite member">
      <form onSubmit={submit} className="space-y-5">
        <div className="flex flex-wrap items-end gap-4">
          <TextField aria-label="Username" value={username} onChange={setUsername} className="min-w-56 flex-1">
            <Input placeholder="Panel username" />
          </TextField>
          <RoleSegment roles={roles} value={role} onChange={chooseRole} disabled={busy} />
        </div>
        <PermissionMatrix catalog={catalog.permissions} value={draft} onChange={setDraft} disabled={busy} />
        <div className="flex items-center justify-between gap-3">
          {error ? <p className="text-sm text-rose">{error.message}</p> : <span />}
          <PrimaryButton type="submit" isDisabled={busy || !username.trim()}>
            Add member
          </PrimaryButton>
        </div>
      </form>
    </Panel>
  );
}
