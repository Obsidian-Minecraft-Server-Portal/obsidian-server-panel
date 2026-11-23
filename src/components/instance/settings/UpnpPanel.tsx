import type { ReactNode } from "react";
import { useUpdateInstance, useUpnp, type InstanceView } from "../../../api/instances";
import { Panel } from "../../ui/Panel";
import { StatusDot } from "../../ui/StatusDot";
import { SwitchRow } from "../../ui/SwitchRow";

export function UpnpPanel({ view }: { view: InstanceView }) {
  const update = useUpdateInstance(view.id);
  const upnp = useUpnp(view.id);
  const info = upnp.data;

  return (
    <Panel title="UPnP">
      <div className="space-y-5">
        <SwitchRow
          label="Forward the server port"
          description="Maps the server port on your router while the server runs."
          isSelected={view.upnp_enabled}
          isDisabled={update.isPending}
          onChange={(upnp_enabled) => update.mutate({ upnp_enabled })}
        />
        <dl className="grid grid-cols-1 gap-3 rounded-control border border-line bg-input/60 p-4 sm:grid-cols-3">
          <Stat label="Mapping">
            <span className="flex items-center gap-2">
              <StatusDot tone={info?.mapped ? "green" : "neutral"} pulse={info?.mapped} />
              {info?.mapped ? "Mapped" : "Not mapped"}
            </span>
          </Stat>
          <Stat label="External IP">{info?.external_ip ?? "—"}</Stat>
          <Stat label="External port">{info?.external_port ?? "—"}</Stat>
        </dl>
        {update.error && <p className="text-sm text-rose">{update.error.message}</p>}
        {upnp.isError && <p className="text-sm text-rose">{upnp.error.message}</p>}
      </div>
    </Panel>
  );
}

function Stat({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="min-w-0">
      <dt className="font-mono text-[10px] tracking-wider text-ink-3 uppercase">{label}</dt>
      <dd className="mt-1 truncate text-sm text-ink-1">{children}</dd>
    </div>
  );
}
