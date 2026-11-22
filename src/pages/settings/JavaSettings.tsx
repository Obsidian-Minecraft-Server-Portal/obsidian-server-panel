import { useCurrentUser } from "../../api/session";
import { InstallRuntimePanel } from "../../components/runtime/InstallRuntimePanel";
import { RuntimeTable } from "../../components/runtime/RuntimeTable";

export function JavaSettings() {
  const { data: user } = useCurrentUser();
  const canManage = user?.is_admin ?? false;

  return (
    <div className="space-y-6">
      <RuntimeTable canManage={canManage} />
      {canManage && <InstallRuntimePanel />}
    </div>
  );
}
