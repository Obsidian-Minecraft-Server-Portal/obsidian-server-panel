import { Button, cn, Tabs } from "@heroui/react";
import type { ContentKind, PlatformId, PlatformInfo } from "../../api/content";
import { KIND_LABELS, KIND_ORDER, PLATFORM_NAMES, PLATFORMS_BY_KIND } from "./meta";

const SOURCE_ORDER: PlatformId[] = [1, 0, 2, 3];

export function isPlatformAvailable(platform: PlatformId, platforms: PlatformInfo[]): boolean {
  return platforms.find((entry) => entry.id === platform)?.available ?? true;
}

export function platformOffersKind(platform: PlatformId, kind: ContentKind): boolean {
  return PLATFORMS_BY_KIND[kind].includes(platform);
}

interface DiscoverTabsProps {
  kind: ContentKind;
  platform: PlatformId;
  onKind: (kind: ContentKind) => void;
}

export function DiscoverTabs({ kind, platform, onKind }: DiscoverTabsProps) {
  return (
    <Tabs selectedKey={kind} onSelectionChange={(key) => onKind(String(key) as ContentKind)}>
      <Tabs.ListContainer>
        <Tabs.List aria-label="Content type">
          {KIND_ORDER.map((entry) => (
            <Tabs.Tab key={entry} id={entry} isDisabled={!platformOffersKind(platform, entry)}>
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
      <span className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">Source</span>
      {SOURCE_ORDER.map((id) => {
        const available = isPlatformAvailable(id, platforms);
        const offered = platformOffersKind(id, kind);
        const usable = available && offered;
        const reason = !available ? "Not configured in this build" : !offered ? `${PLATFORM_NAMES[id]} has no ${KIND_LABELS[kind].toLowerCase()}` : undefined;
        return (
          <span key={id} title={reason} className="inline-flex">
            <Button
              variant="ghost"
              isDisabled={!usable}
              onPress={() => onPlatform(id)}
              aria-pressed={id === platform}
              className={cn(
                "h-auto rounded-full border px-3.5 py-1.5 text-sm font-medium transition",
                id === platform
                  ? "border-line-strong bg-raised text-ink-0"
                  : "border-line bg-panel text-ink-2 hover:border-line-strong hover:text-ink-0",
                !usable && "cursor-not-allowed opacity-40 hover:border-line hover:text-ink-2",
              )}
            >
              {PLATFORM_NAMES[id]}
            </Button>
          </span>
        );
      })}
    </div>
  );
}
