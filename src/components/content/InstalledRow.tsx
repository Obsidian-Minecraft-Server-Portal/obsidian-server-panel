import { Switch, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { ContentUpdate, InstalledContent } from "../../api/content";
import { contentIconUrl } from "../../api/content";
import { Chip } from "../ui/Chip";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { PLATFORM_NAMES } from "./meta";
import { ProjectIcon } from "./ProjectIcon";

interface InstalledRowProps {
  item: InstalledContent;
  update: ContentUpdate | undefined;
  busy: boolean;
  confirmingRemove: boolean;
  onToggle: (enabled: boolean) => void;
  onApplyUpdate: (update: ContentUpdate) => void;
  onRemove: () => void;
}

const DISABLED_SUFFIX = /\.disabled$/;

export function InstalledRow({ item, update, busy, confirmingRemove, onToggle, onApplyUpdate, onRemove }: InstalledRowProps) {
  const displayName = item.filename.replace(DISABLED_SUFFIX, "");

  return (
    <div className="flex flex-wrap items-center gap-4 rounded-panel border border-line bg-panel p-3.5 transition hover:border-line-strong">
      <ProjectIcon src={contentIconUrl(item.instance_id, item.id)} alt="" className="size-12" />
      <div className="min-w-0 flex-1 basis-56 space-y-1.5">
        <p className={cn("truncate font-medium text-ink-0", !item.enabled && "text-ink-3 line-through")}>{displayName}</p>
        <div className="flex flex-wrap items-center gap-1.5">
          <Chip>{item.kind === "plugin" ? "Plugin" : "Mod"}</Chip>
          {item.platform !== null && <Chip tone="cyan">{PLATFORM_NAMES[item.platform]}</Chip>}
          {update && (
            <Chip tone="amber" pulse>
              Update · {update.latest.version_number}
            </Chip>
          )}
        </div>
      </div>
      <div className="flex items-center gap-3">
        {update && (
          <PrimaryButton size="sm" isDisabled={busy} onPress={() => onApplyUpdate(update)}>
            Update
          </PrimaryButton>
        )}
        <Switch
          aria-label={item.enabled ? "Disable" : "Enable"}
          isSelected={item.enabled}
          isDisabled={busy}
          onChange={onToggle}
        >
          <Switch.Control>
            <Switch.Thumb />
          </Switch.Control>
        </Switch>
        <GhostButton
          size="sm"
          isDisabled={busy}
          onPress={onRemove}
          className={cn(confirmingRemove && "border-rose/50 text-rose")}
        >
          <Icon icon={confirmingRemove ? "lucide:triangle-alert" : "lucide:trash-2"} width={15} height={15} />
          {confirmingRemove ? "Confirm" : "Remove"}
        </GhostButton>
      </div>
    </div>
  );
}
