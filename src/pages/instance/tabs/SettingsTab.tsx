import { useInstance } from "../../../api/instances";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { DangerZone } from "../../../components/instance/settings/DangerZone";
import { GeneralSettings, generalSettingsKey } from "../../../components/instance/settings/GeneralSettings";
import { IconPanel } from "../../../components/instance/settings/IconPanel";
import { UpnpPanel } from "../../../components/instance/settings/UpnpPanel";

export function SettingsTab() {
  const id = useInstanceId();
  const instance = useInstance(id);

  if (instance.isError) return <p className="px-8 text-sm text-rose">{instance.error.message}</p>;
  if (!instance.data) return <p className="px-8 text-sm text-ink-3">Loading settings…</p>;
  const view = instance.data;

  return (
    <div className="grid gap-5 px-8 pb-8 xl:grid-cols-2">
      <GeneralSettings key={generalSettingsKey(view)} view={view} />
      <div className="space-y-5">
        <UpnpPanel view={view} />
        <IconPanel view={view} />
        <DangerZone view={view} />
      </div>
    </div>
  );
}
