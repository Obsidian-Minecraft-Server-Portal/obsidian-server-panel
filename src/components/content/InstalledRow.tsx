import { Switch, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import copyToClipboard from "copy-to-clipboard";
import type { ContentUpdate, InstalledContent } from "../../api/content";
import { contentIconUrl } from "../../api/content";
import { ContextMenu, MenuButton, type MenuAction } from "../ui/ContextMenu";
import { Chip } from "../ui/Chip";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { stopRowClick } from "../ui/openUnlessInteractive";
import { useContextMenu } from "../ui/useContextMenu";
import { PLATFORM_NAMES } from "./meta";
import { ProjectIcon } from "./ProjectIcon";
import type { RelationsTab } from "./RelationsModal";

interface InstalledRowProps {
  item: InstalledContent;
  update: ContentUpdate | undefined;
  busy: boolean;
  confirmingRemove: boolean;
  onToggle: (enabled: boolean) => void;
  onApplyUpdate: (update: ContentUpdate) => void;
  onRemove: () => void;
  onRelations: (tab: RelationsTab) => void;
  onAssociate: () => void;
  onSwitchVersion: () => void;
}

const DISABLED_SUFFIX = /\.disabled$/;

export function InstalledRow({
  item,
  update,
  busy,
  confirmingRemove,
  onToggle,
  onApplyUpdate,
  onRemove,
  onRelations,
  onAssociate,
  onSwitchVersion,
}: InstalledRowProps) {
  const filename = item.filename.replace(DISABLED_SUFFIX, "");
  const title = item.name ?? filename;
  const menu = useContextMenu();

  const actions: MenuAction[] = [
    {
      key: "toggle",
      label: item.enabled ? "Disable" : "Enable",
      icon: item.enabled ? "lucide:pause" : "lucide:play",
      run: () => onToggle(!item.enabled),
    },
    ...(update
      ? [
          {
            key: "update",
            label: `Update to ${update.latest.version_number}`,
            icon: "lucide:arrow-up-circle",
            run: () => onApplyUpdate(update),
          },
        ]
      : []),
    ...(item.platform !== null && item.project_id !== null
      ? [{ key: "versions", label: "Switch version…", icon: "lucide:history", run: onSwitchVersion }]
      : []),
    { key: "dependencies", label: "Show dependencies", icon: "lucide:list-tree", run: () => onRelations("dependencies") },
    { key: "dependents", label: "Show dependents", icon: "lucide:network", run: () => onRelations("dependents") },
    { key: "associate", label: "Associate with…", icon: "lucide:link", run: onAssociate },
    { key: "copy", label: "Copy filename", icon: "lucide:copy", run: () => void copyToClipboard(filename, { fallbackToPrompt: true }) },
    {
      key: "remove",
      label: confirmingRemove ? "Confirm remove" : "Remove",
      icon: "lucide:trash-2",
      danger: true,
      run: onRemove,
    },
  ];

  return (
    <div
      onClick={(event) => menu.openAt(event.clientX, event.clientY)}
      onContextMenu={(event) => {
        event.preventDefault();
        menu.openAt(event.clientX, event.clientY);
      }}
      className="flex min-w-0 cursor-pointer flex-wrap items-center gap-3 rounded-panel border border-line bg-panel p-3.5 transition hover:border-line-strong hover:bg-raised sm:gap-4"
    >
      <ProjectIcon src={contentIconUrl(item.instance_id, item.id)} alt="" className="size-12" />
      <div className="min-w-0 flex-1 basis-56 space-y-1">
        <p className={cn("truncate font-medium text-ink-0", !item.enabled && "text-ink-3 line-through")}>{title}</p>
        {item.name && <p className="truncate font-mono text-[11px] text-ink-3">{filename}</p>}
        <div className="flex flex-wrap items-center gap-1.5 pt-0.5">
          <Chip>{item.kind === "plugin" ? "Plugin" : item.kind === "datapack" ? "Data pack" : "Mod"}</Chip>
          {item.platform !== null ? (
            <Chip tone="secondary">{PLATFORM_NAMES[item.platform]}</Chip>
          ) : (
            <Chip tone="amber">Not linked</Chip>
          )}
          {update && (
            <Chip tone="amber" pulse>
              Update · {update.latest.version_number}
            </Chip>
          )}
        </div>
      </div>
      <div className="flex max-w-full flex-wrap items-center gap-2 sm:gap-3" onClick={stopRowClick}>
        {update && (
          <PrimaryButton size="sm" isDisabled={busy} onPress={() => onApplyUpdate(update)}>
            Update
          </PrimaryButton>
        )}
        <Switch isSelected={item.enabled} isDisabled={busy} onChange={onToggle}>
          <Switch.Content aria-label={item.enabled ? "Disable" : "Enable"}>
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
          </Switch.Content>
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
        <MenuButton label={`${title} actions`} onOpen={menu.openAt} />
      </div>
      {menu.menu && (
        <span onClick={stopRowClick}>
          <ContextMenu
            x={menu.menu.x}
            y={menu.menu.y}
            label={`${title} actions`}
            actions={actions}
            onClose={menu.close}
          />
        </span>
      )}
    </div>
  );
}
