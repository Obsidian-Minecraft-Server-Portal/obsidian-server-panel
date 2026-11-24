import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { useCurrentWorld } from "../../api/content";
import { useResourcePacks } from "../../api/resourcePacks";
import { GhostButton } from "../ui/GhostButton";
import { ResourcePacksModal } from "./ResourcePacksModal";

export function ServerWorldPanel({
  instanceId,
  canInstall,
  canRemove,
}: {
  instanceId: number;
  canInstall: boolean;
  canRemove: boolean;
}) {
  const world = useCurrentWorld(instanceId);
  const packs = useResourcePacks(instanceId);
  const [managing, setManaging] = useState(false);
  const list = packs.data?.packs ?? [];
  const top = list[0];

  return (
    <div className="grid grid-cols-1 divide-y divide-line overflow-hidden rounded-panel border border-line bg-panel md:grid-cols-2 md:divide-x md:divide-y-0">
      <div className="flex items-center gap-3 px-4 py-3">
        <Icon icon="lucide:globe" width={18} height={18} className="text-ink-3" />
        <div className="min-w-0">
          <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">World</p>
          <p className="truncate font-mono text-sm text-ink-1">{world.data?.level_name ?? "…"}</p>
        </div>
      </div>
      <div className="flex items-center gap-3 px-4 py-3">
        <Icon icon="lucide:palette" width={18} height={18} className="text-ink-3" />
        <div className="min-w-0 flex-1">
          <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">Server resource packs</p>
          <p className="truncate text-sm text-ink-1">
            {top ? (
              <>
                {list.length} {list.length === 1 ? "pack" : "packs"} · top: {top.name}
              </>
            ) : (
              <span className="text-ink-3">None</span>
            )}
          </p>
        </div>
        <GhostButton size="sm" onPress={() => setManaging(true)}>
          Manage
        </GhostButton>
      </div>
      <ResourcePacksModal
        instanceId={instanceId}
        open={managing}
        canInstall={canInstall}
        canRemove={canRemove}
        onClose={() => setManaging(false)}
      />
    </div>
  );
}
