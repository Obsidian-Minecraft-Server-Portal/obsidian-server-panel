import { hasPermission, useInstance } from "../../../api/instances";
import { useMaps } from "../../../api/maps";
import { useInstanceId } from "../../../components/instance/useInstanceId";
import { BannerPanel } from "../../../components/instance/settings/BannerPanel";
import { DangerZone } from "../../../components/instance/settings/DangerZone";
import { GeneralSettings } from "../../../components/instance/settings/GeneralSettings";
import { IconPanel } from "../../../components/instance/settings/IconPanel";
import { MapSettingsCard } from "../../../components/instance/settings/MapSettingsCard";
import { SoftwarePanel } from "../../../components/instance/settings/SoftwarePanel";
import { UpnpPanel } from "../../../components/instance/settings/UpnpPanel";
import { MembersSection } from "./MembersTab";

export function SettingsTab() {
  const id = useInstanceId();
  const instance = useInstance(id);
  const maps = useMaps(id);

  if (instance.isError) return <p className="px-4 sm:px-8 text-sm text-rose">{instance.error.message}</p>;
  if (!instance.data) return <p className="px-4 sm:px-8 text-sm text-ink-3">Loading settings…</p>;
  const view = instance.data;

  return (
    <div className="space-y-5 px-4 sm:px-8 pb-8">
      <GeneralSettings view={view} />
      <UpnpPanel view={view} />
      {(maps.data ?? []).map((map) => (
        <MapSettingsCard
          key={`${map.kind}-${map.settings.enabled}-${map.settings.port}-${map.settings.bind ?? ""}`}
          instanceId={id}
          map={map}
        />
      ))}
      <div className="grid grid-cols-1 gap-5 xl:grid-cols-2">
        <SoftwarePanel view={view} />
        {hasPermission(view, "members.manage") && <MembersSection instanceId={id} />}
        <IconPanel view={view} />
        <BannerPanel view={view} />
        <DangerZone view={view} />
      </div>
    </div>
  );
}
