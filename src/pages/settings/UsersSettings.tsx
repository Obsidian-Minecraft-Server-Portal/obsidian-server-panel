import { Icon } from "@iconify-icon/react";
import { useOverlayState } from "@heroui/react";
import { useState } from "react";
import type { UserProfile } from "../../api/auth";
import { Panel } from "../../components/ui/Panel";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { InvitesPanel } from "../users/InvitesPanel";
import { UserDialog } from "../users/UserDialog";
import { UsersTable } from "../users/UsersTable";

export function UsersSettings() {
  const dialog = useOverlayState();
  const [editing, setEditing] = useState<UserProfile | null>(null);

  const openNew = () => {
    setEditing(null);
    dialog.open();
  };

  const openEdit = (user: UserProfile) => {
    setEditing(user);
    dialog.open();
  };

  return (
    <>
      <div className="space-y-6">
        <Panel
          title="Accounts"
          actions={
            <PrimaryButton onPress={openNew}>
              <Icon icon="lucide:user-plus" width={16} height={16} />
              New user
            </PrimaryButton>
          }
        >
          <UsersTable onEdit={openEdit} />
        </Panel>
        <InvitesPanel />
      </div>
      <UserDialog state={dialog} user={editing} />
    </>
  );
}
