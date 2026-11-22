import { Tabs, ToggleButton, ToggleButtonGroup } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { ContentKind, PlatformId, PlatformInfo } from "../../api/content";
import { useMinecraftVersions, type LoaderKind } from "../../api/loaders";
import { OptionSelect, type SelectOption } from "../runtime/OptionSelect";
import { LOADER_META } from "../runtime/loaderMeta";
import { KIND_LABELS, KIND_ORDER, LOADERS_BY_KIND, PLATFORM_NAMES, PLATFORMS_BY_KIND } from "./meta";
import type { ResultView } from "./ProjectResults";

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

interface ViewToggleProps {
  view: ResultView;
  onView: (view: ResultView) => void;
}

const VIEWS: { id: ResultView; icon: string; label: string }[] = [
  { id: "grid", icon: "lucide:layout-grid", label: "Grid view" },
  { id: "compact", icon: "lucide:list", label: "Compact view" },
  { id: "table", icon: "lucide:table-2", label: "Table view" },
];

export function ViewToggle({ view, onView }: ViewToggleProps) {
  return (
    <ToggleButtonGroup
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={new Set([view])}
      onSelectionChange={(keys) => {
        const next = [...keys][0];
        if (next !== undefined) onView(String(next) as ResultView);
      }}
      className="rounded-control border border-line-strong bg-input p-0.5"
    >
      {VIEWS.map((entry) => (
        <ToggleButton key={entry.id} id={entry.id} aria-label={entry.label} className="size-9 rounded-control p-0 text-ink-3 data-[selected=true]:bg-accent-tint data-[selected=true]:text-accent">
          <Icon icon={entry.icon} width={16} height={16} />
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
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
