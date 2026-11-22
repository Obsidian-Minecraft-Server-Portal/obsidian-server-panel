import { Button, Modal, type UseOverlayStateReturn } from "@heroui/react";
import { useState, type FormEvent } from "react";
import type { UserProfile } from "../../api/auth";
import { useCreateUser, useDeleteUser, useSetUserPassword, useUpdateUser } from "../../api/users";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { GhostButton } from "../../components/ui/GhostButton";
import { FormField } from "../account/FormField";
import { SwitchRow } from "../../components/ui/SwitchRow";
import { errorMessage } from "../account/errorMessage";

interface UserDialogProps {
  state: UseOverlayStateReturn;
  user: UserProfile | null;
}

export function UserDialog({ state, user }: UserDialogProps) {
  return (
    <Modal state={state}>
      <Modal.Backdrop>
        <Modal.Container>
          <Modal.Dialog className="rounded-card border border-line-strong bg-panel sm:max-w-md">
            <Modal.Header>
              <Modal.Heading className="font-display text-xl font-bold text-ink-0">
                {user ? `Edit ${user.username}` : "New user"}
              </Modal.Heading>
            </Modal.Header>
            <UserForm key={user?.id ?? "new"} user={user} onClose={state.close} />
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>
    </Modal>
  );
}

function parseLimit(value: string): number | undefined | null {
  const trimmed = value.trim();
  if (trimmed === "") return undefined;
  const parsed = Number(trimmed);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
}

function UserForm({ user, onClose }: { user: UserProfile | null; onClose: () => void }) {
  const create = useCreateUser();
  const update = useUpdateUser();
  const remove = useDeleteUser();
  const setUserPassword = useSetUserPassword();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState(user?.email ?? "");
  const [password, setPassword] = useState("");
  const [isAdmin, setIsAdmin] = useState(user?.is_admin ?? false);
  const [limit, setLimit] = useState(user ? String(user.instance_limit) : "");
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const saving = create.isPending || update.isPending || setUserPassword.isPending;
  const pending = saving || remove.isPending;

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const instanceLimit = parseLimit(limit);
    if (instanceLimit === null) {
      setError("The instance limit must be a whole number.");
      return;
    }
    setError(null);
    try {
      if (user) {
        await update.mutateAsync({
          id: user.id,
          body: { email: email.trim(), is_admin: isAdmin, instance_limit: instanceLimit },
        });
        if (password) await setUserPassword.mutateAsync({ id: user.id, password });
      } else {
        await create.mutateAsync({
          username: username.trim(),
          email: email.trim() || undefined,
          password,
          is_admin: isAdmin,
          instance_limit: instanceLimit,
        });
      }
      onClose();
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  const destroy = () => {
    if (!user) return;
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    remove.mutate(user.id, { onSuccess: onClose, onError: (err) => setError(errorMessage(err)) });
  };

  return (
    <form onSubmit={submit} className="flex flex-col">
      <Modal.Body className="space-y-5">
        {!user && (
          <>
            <FormField label="Username" value={username} onChange={setUsername} autoComplete="off" autoFocus isRequired />
            <FormField label="Temporary password" type="password" value={password} onChange={setPassword} autoComplete="new-password" isRequired />
          </>
        )}
        {user && (
          <FormField
            label="New password"
            type="password"
            value={password}
            onChange={setPassword}
            autoComplete="new-password"
            description="Optional. Replaces the password and signs the user out everywhere."
          />
        )}
        <FormField
          label="Email"
          type="email"
          value={email}
          onChange={setEmail}
          autoComplete="off"
          description={user ? "Leave blank to remove the address." : "Optional."}
        />
        <FormField
          label="Instance limit"
          inputMode="numeric"
          value={limit}
          onChange={setLimit}
          placeholder="Server default"
          description="How many instances this user may create. Leave blank for the default."
        />
        <SwitchRow
          label="Administrator"
          description="Administrators manage users, settings and every instance."
          isSelected={isAdmin}
          onChange={setIsAdmin}
        />
        {error && (
          <p role="alert" className="text-sm text-rose">
            {error}
          </p>
        )}
      </Modal.Body>
      <Modal.Footer className="flex flex-wrap items-center justify-between gap-3">
        {user ? (
          <Button variant="ghost" onPress={destroy} isPending={remove.isPending} className="rounded-control text-rose">
            {confirmDelete ? "Confirm delete" : "Delete user"}
          </Button>
        ) : (
          <span />
        )}
        <div className="flex gap-2">
          <GhostButton onPress={onClose} isDisabled={pending}>
            Cancel
          </GhostButton>
          <PrimaryButton type="submit" isPending={saving}>
            {user ? "Save changes" : "Create user"}
          </PrimaryButton>
        </div>
      </Modal.Footer>
    </form>
  );
}
