import { Tabs } from "@heroui/react";
import type { ContentKind, PlatformId, PlatformInfo } from "../../api/content";
import { useMinecraftVersions, type LoaderKind } from "../../api/loaders";
import { OptionSelect, type SelectOption } from "../runtime/OptionSelect";
import { LOADER_META } from "../runtime/loaderMeta";
import { KIND_LABELS, KIND_ORDER, LOADERS_BY_KIND, PLATFORM_NAMES, PLATFORMS_BY_KIND } from "./meta";

interface DiscoverTabsProps {
  kind: ContentKind;
  onKind: (kind: ContentKind) => void;
}

export function DiscoverTabs({ kind, onKind }: DiscoverTabsProps) {
  return (
    <Tabs selectedKey={kind} onSelectionChange={(key) => onKind(String(key) as ContentKind)}>
      <Tabs.ListContainer>
        <Tabs.List aria-label="Content type">
          {KIND_ORDER.map((entry) => (
            <Tabs.Tab key={entry} id={entry}>
              {KIND_LABELS[entry]}
              <Tabs.Indicator />
            </Tabs.Tab>
          ))}
        </Tabs.List>
      </Tabs.ListContainer>
    </Tabs>
  );
}

const ANY = "any";

function fromAny(value: string | null): string | null {
  return value === ANY ? null : value;
}

interface DiscoverSelectsProps {
  kind: ContentKind;
  platform: PlatformId;
  platforms: PlatformInfo[];
  onPlatform: (platform: PlatformId) => void;
  loader: LoaderKind | null;
  onLoader: (loader: LoaderKind | null) => void;
  mcVersion: string | null;
  onMcVersion: (version: string | null) => void;
}

export function DiscoverSelects({
  kind,
  platform,
  platforms,
  onPlatform,
  loader,
  onLoader,
  mcVersion,
  onMcVersion,
}: DiscoverSelectsProps) {
  const { data: versions } = useMinecraftVersions("vanilla");

  const platformOptions: SelectOption[] = PLATFORMS_BY_KIND[kind].map((id) => {
    const info = platforms.find((entry) => entry.id === id);
    const available = info?.available ?? true;
    return { id: String(id), label: available ? PLATFORM_NAMES[id] : `${PLATFORM_NAMES[id]} (not configured)` };
  });
  const loaderOptions: SelectOption[] = [
    { id: ANY, label: "Any loader" },
    ...LOADERS_BY_KIND[kind].map((id) => ({ id, label: LOADER_META[id].label })),
  ];
  const versionOptions: SelectOption[] = [
    { id: ANY, label: "Any version" },
    ...(versions ?? []).map((version) => ({ id: version, label: version })),
  ];

  return (
    <div className="flex flex-wrap items-end gap-3">
      <OptionSelect
        label="Platform"
        className="w-44"
        options={platformOptions}
        value={String(platform)}
        onChange={(value) => value !== null && onPlatform(Number(value) as PlatformId)}
      />
      {kind !== "modpack" && (
        <>
          <OptionSelect
            label="Loader"
            className="w-44"
            options={loaderOptions}
            value={loader ?? ANY}
            onChange={(value) => onLoader(fromAny(value) as LoaderKind | null)}
          />
          <OptionSelect
            label="Minecraft version"
            className="w-40"
            options={versionOptions}
            value={mcVersion ?? ANY}
            onChange={(value) => onMcVersion(fromAny(value))}
          />
        </>
      )}
    </div>
  );
}
