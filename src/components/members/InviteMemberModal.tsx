import { Modal } from "@heroui/react";
import { useState, type FormEvent } from "react";
import { useAddMember, useMemberCandidates, useUpdateMember, type PermissionCatalog } from "../../api/members";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { SearchSelect, type SearchOption } from "../ui/SearchSelect";
import { useDebouncedValue } from "../ui/useDebouncedValue";
import { PermissionMatrix } from "./PermissionMatrix";
import { RoleSegment } from "./RoleSegment";
import { sameMembers } from "./format";

const MIN_SEARCH_LENGTH = 2;
const SEARCH_DEBOUNCE_MS = 250;

interface InviteMemberModalProps {
  instanceId: number;
  catalog: PermissionCatalog;
  onClose: () => void;
}

export function InviteMemberModal({ instanceId, catalog, onClose }: InviteMemberModalProps) {
  const roles = catalog.roles.map((preset) => preset.role);
  const presetFor = (role: string) => catalog.roles.find((preset) => preset.role === role)?.permissions ?? [];

  const [search, setSearch] = useState("");
  const [username, setUsername] = useState<string | null>(null);
  const [role, setRole] = useState(roles[0] ?? "viewer");
  const [draft, setDraft] = useState(() => presetFor(role));
  const debouncedSearch = useDebouncedValue(search, SEARCH_DEBOUNCE_MS);
  const candidates = useMemberCandidates(instanceId, debouncedSearch);
  const add = useAddMember(instanceId);
  const update = useUpdateMember(instanceId);
  const busy = add.isPending || update.isPending;
  const error = add.error ?? update.error;

  const names = candidates.data ?? [];
  const pool = username !== null && !names.includes(username) ? [username, ...names] : names;
  const options: SearchOption[] = pool.map((name) => ({ id: name, label: name }));
  const searchable = debouncedSearch.trim().length >= MIN_SEARCH_LENGTH;

  const chooseRole = (next: string) => {
    setRole(next);
    setDraft(presetFor(next));
  };

  const invite = async () => {
    if (username === null) return;
    update.reset();
    const member = await add.mutateAsync({ username, role }).catch(() => null);
    if (!member) return;
    if (sameMembers(draft, member.permissions)) {
      onClose();
      return;
    }
    update.mutate({ userId: member.user_id, permissions: draft }, { onSuccess: onClose });
  };

  return (
    <Modal.Backdrop isOpen onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="lg">
        <Modal.Dialog className="bg-panel text-ink-1">
          <Modal.Header className="flex flex-col items-start gap-1">
            <Modal.Heading className="text-lg font-semibold text-ink-0">Invite member</Modal.Heading>
            <p className="text-sm text-ink-3">Pick a panel user and a role, then adjust what they can do.</p>
          </Modal.Header>
          <Modal.Body>
            <form
              className="space-y-5"
              onSubmit={(event: FormEvent) => {
                event.preventDefault();
                void invite();
              }}
            >
              <div className="flex flex-wrap items-end gap-4">
                <SearchSelect
                  label="Panel username"
                  isRequired
                  options={options}
                  value={username}
                  onChange={setUsername}
                  onSearch={setSearch}
                  placeholder="Type a username"
                  emptyText={searchable ? "No panel user matches" : "Type at least two letters"}
                  isDisabled={busy}
                  className="min-w-56 flex-1"
                />
                <RoleSegment roles={roles} value={role} onChange={chooseRole} disabled={busy} />
              </div>
              <PermissionMatrix catalog={catalog.permissions} value={draft} onChange={setDraft} disabled={busy} />
              {error && <p className="text-sm text-rose">{error.message}</p>}
            </form>
          </Modal.Body>
          <Modal.Footer className="flex justify-end gap-2">
            <GhostButton onPress={onClose}>Cancel</GhostButton>
            <PrimaryButton isPending={busy} isDisabled={busy || username === null} onPress={() => void invite()}>
              Add member
            </PrimaryButton>
          </Modal.Footer>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}
