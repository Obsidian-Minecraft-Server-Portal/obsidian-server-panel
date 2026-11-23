import { Button, cn, Tabs } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { ContentKind, PlatformId, PlatformInfo } from "../../api/content";
import { KIND_LABELS, KIND_ORDER, PLATFORM_ICONS, PLATFORM_NAMES, SOURCE_ORDER, platformOffersKind } from "./meta";

export function isPlatformAvailable(platform: PlatformId, platforms: PlatformInfo[]): boolean {
  return platforms.find((entry) => entry.id === platform)?.available ?? true;
}

interface DiscoverTabsProps {
  kind: ContentKind;
  platform: PlatformId;
  platforms: PlatformInfo[];
  onKind: (kind: ContentKind) => void;
}

export function DiscoverTabs({ kind, platform, platforms, onKind }: DiscoverTabsProps) {
  return (
    <Tabs variant="secondary" selectedKey={kind} onSelectionChange={(key) => onKind(String(key) as ContentKind)}>
      <Tabs.ListContainer>
        <Tabs.List aria-label="Content type" className="w-full">
          {KIND_ORDER.map((entry) => (
            <Tabs.Tab
              key={entry}
              id={entry}
              isDisabled={!platformOffersKind(platforms, platform, entry)}
              className="flex-1 whitespace-nowrap font-medium text-ink-2 data-[disabled=true]:text-ink-4 data-[selected=true]:text-accent"
            >
              {KIND_LABELS[entry]}
              <Tabs.Indicator />
            </Tabs.Tab>
          ))}
        </Tabs.List>
      </Tabs.ListContainer>
    </Tabs>
  );
}

interface SourceChipsProps {
  kind: ContentKind;
  platform: PlatformId;
  platforms: PlatformInfo[];
  onPlatform: (platform: PlatformId) => void;
}

export function SourceChips({ kind, platform, platforms, onPlatform }: SourceChipsProps) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="mr-1 font-mono text-[11px] tracking-wider text-ink-3 uppercase">Source</span>
      {SOURCE_ORDER.map((id) => {
        const available = isPlatformAvailable(id, platforms);
        const offered = platformOffersKind(platforms, id, kind);
        const selected = id === platform;
        const reason = !available
          ? "Not configured in this build"
          : !offered
            ? `Switches to `
            : undefined;
        return (
          <span key={id} title={reason} className="inline-flex">
            <Button
              variant="ghost"
              isDisabled={!available}
              onPress={() => onPlatform(id)}
              aria-pressed={selected}
              className={cn(
                "h-8 gap-2 rounded-full border px-3 text-sm font-medium transition-colors",
                selected
                  ? "border-accent/50 bg-accent-tint text-accent"
                  : "border-line-strong bg-transparent text-ink-2 hover:bg-raised hover:text-ink-0",
                !available && "cursor-not-allowed opacity-40 hover:bg-transparent hover:text-ink-2",
              )}
            >
              <Icon icon={PLATFORM_ICONS[id]} width={15} height={15} />
              {PLATFORM_NAMES[id]}
            </Button>
          </span>
        );
      })}
    </div>
  );
}
