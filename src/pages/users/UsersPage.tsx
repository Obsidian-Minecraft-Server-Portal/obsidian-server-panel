import { Icon } from "@iconify-icon/react";
import { useOverlayState } from "@heroui/react";
import { useState } from "react";
import type { UserProfile } from "../../api/auth";
import { PageHeader } from "../../components/ui/PageHeader";
import { Panel } from "../../components/ui/Panel";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { InvitesPanel } from "./InvitesPanel";
import { UserDialog } from "./UserDialog";
import { UsersTable } from "./UsersTable";

export function UsersPage() {
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
      <PageHeader
        title="Users"
        subtitle="Accounts, instance limits and invite links."
        actions={
          <PrimaryButton onPress={openNew}>
            <Icon icon="lucide:user-plus" width={16} height={16} />
            New user
          </PrimaryButton>
        }
      />
      <div className="grid gap-6 px-8 pb-8 xl:grid-cols-[1fr_24rem]">
        <Panel title="Accounts">
          <UsersTable onEdit={openEdit} />
        </Panel>
        <InvitesPanel />
      </div>
      <UserDialog state={dialog} user={editing} />
    </>
  );
}
