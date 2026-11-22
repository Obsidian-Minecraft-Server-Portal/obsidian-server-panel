import { LOADER_META } from "../runtime/loaderMeta";
import { Chip } from "../ui/Chip";
import type { InstalledContent } from "../../api/content";
import type { InstanceView } from "../../api/instances";
import { isPulsing, statusLabel, statusTone } from "./status";

export function StatusChip({ instance }: { instance: Pick<InstanceView, "status"> }) {
  return (
    <Chip tone={statusTone(instance.status)} pulse={isPulsing(instance.status)}>
      {statusLabel(instance.status)}
    </Chip>
  );
}

export function LoaderChip({ instance }: { instance: Pick<InstanceView, "loader" | "loader_version" | "mc_version"> }) {
  return (
    <Chip tone="secondary">
      {LOADER_META[instance.loader].label}
      {instance.loader_version ? ` ${instance.loader_version}` : ""}
    </Chip>
  );
}

export function VersionChip({ mcVersion }: { mcVersion: string }) {
  return <Chip>MC {mcVersion}</Chip>;
}

export function ContentChip({ items }: { items: Pick<InstalledContent, "kind">[] }) {
  const plugins = items.filter((item) => item.kind === "plugin").length;
  const mods = items.length - plugins;
  return <Chip>{plugins > mods ? `${plugins} PLUGINS` : `${mods} MODS`}</Chip>;
}
