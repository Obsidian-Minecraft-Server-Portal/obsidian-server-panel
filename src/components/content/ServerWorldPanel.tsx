import { Icon } from "@iconify-icon/react";
import { useClearResourcePack, useCurrentWorld, useResourcePack } from "../../api/content";
import { GhostButton } from "../ui/GhostButton";

function packName(url: string): string {
  try {
    return decodeURIComponent(new URL(url).pathname.split("/").pop() ?? url);
  } catch {
    return url;
  }
}

export function ServerWorldPanel({ instanceId, canRemove }: { instanceId: number; canRemove: boolean }) {
  const world = useCurrentWorld(instanceId);
  const pack = useResourcePack(instanceId);
  const clear = useClearResourcePack(instanceId);

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
          <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">Server resource pack</p>
          <p className="truncate text-sm text-ink-1" title={pack.data?.url}>
            {pack.data ? packName(pack.data.url) : <span className="text-ink-3">None</span>}
          </p>
        </div>
        {pack.data && canRemove && (
          <GhostButton size="sm" isDisabled={clear.isPending} onPress={() => clear.mutate()}>
            Remove
          </GhostButton>
        )}
      </div>
    </div>
  );
}
