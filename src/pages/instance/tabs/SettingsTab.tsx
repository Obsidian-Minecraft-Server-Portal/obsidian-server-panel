import { hasPermission, useInstance } from "../../../api/instances";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { DangerZone } from "../../../components/instance/settings/DangerZone";
import { GeneralSettings, generalSettingsKey } from "../../../components/instance/settings/GeneralSettings";
import { SoftwarePanel, softwareKey } from "../../../components/instance/settings/SoftwarePanel";
import { IconPanel } from "../../../components/instance/settings/IconPanel";
import { UpnpPanel } from "../../../components/instance/settings/UpnpPanel";
import { MembersSection } from "./MembersTab";

export function SettingsTab() {
  const id = useInstanceId();
  const instance = useInstance(id);

  if (instance.isError) return <p className="px-8 text-sm text-rose">{instance.error.message}</p>;
  if (!instance.data) return <p className="px-8 text-sm text-ink-3">Loading settings…</p>;
  const view = instance.data;

  return (
    <div className="grid gap-5 px-8 pb-8 xl:grid-cols-2">
      <div className="space-y-5">
        <GeneralSettings key={generalSettingsKey(view)} view={view} />
        <SoftwarePanel key={softwareKey(view)} view={view} />
      </div>
      <div className="space-y-5">
        <UpnpPanel view={view} />
        <IconPanel view={view} />
        <DangerZone view={view} />
      </div>
      {hasPermission(view, "members.manage") && (
        <div className="xl:col-span-2">
          <MembersSection instanceId={id} />
        </div>
      )}
    </div>
  );
}
