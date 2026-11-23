import { Button, SearchField, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { createContext, useContext, useState, type ReactNode } from "react";
import type { CategoryGroup, ContentFilters, ContentKind, Facets, SideFilter } from "../../api/content";
import { useMinecraftVersions, type LoaderKind } from "../../api/loaders";
import { LOADER_META } from "../runtime/loaderMeta";
import { CheckboxRow } from "../ui/CheckboxRow";
import { LOADERS_BY_KIND } from "./meta";

const SHOWN_CATEGORIES = 8;
const SHOWN_VERSIONS = 10;

const SIDES: { id: SideFilter; label: string }[] = [
  { id: "client", label: "Client" },
  { id: "server", label: "Server" },
];

interface DiscoverSidebarProps {
  kind: ContentKind;
  platformName: string;
  facets: Facets | undefined;
  filters: ContentFilters;
  loader: LoaderKind | null;
  mcVersion: string | null;
  onLoader: (loader: LoaderKind | null) => void;
  onToggleExcludedLoader: (loader: LoaderKind) => void;
  onMcVersion: (version: string | null) => void;
  onToggleCategory: (id: string, multiple: boolean) => void;
  onToggleExcluded: (id: string) => void;
  onToggleEnvironment: (side: SideFilter) => void;
  onOpenSource: (on: boolean) => void;
  onClear: () => void;
}

const SectionDisabled = createContext(false);

export function DiscoverSidebar(props: DiscoverSidebarProps) {
  const { kind, facets, filters, loader, mcVersion, platformName } = props;
  const hasPicks =
    loader !== null ||
    mcVersion !== null ||
    filters.categories.length > 0 ||
    filters.excludedCategories.length > 0 ||
    filters.excludedLoaders.length > 0 ||
    filters.environments.length > 0 ||
    filters.openSource;
  const loading = facets === undefined;
  const hasGroups = (facets?.categories.length ?? 0) > 0;
  const canExclude = facets?.exclusions ?? false;

  return (
    <div className="flex flex-col gap-2.5">
      <Section
        title="Game version"
        disabled={!facets?.game_versions}
        hint={loading ? "Loading…" : `Not offered by ${platformName}`}
      >
        <VersionList selected={mcVersion} onSelect={props.onMcVersion} />
      </Section>
      {LOADERS_BY_KIND[kind].length > 0 && (
        <Section title="Loader" disabled={!facets?.loaders} hint={loading ? "Loading…" : `Not offered by ${platformName}`}>
          {pinFirst(LOADERS_BY_KIND[kind], (id) => loader === id || filters.excludedLoaders.includes(id)).map((id) => (
            <FilterRow
              key={id}
              label={LOADER_META[id].label}
              picked={loader === id}
              excluded={filters.excludedLoaders.includes(id)}
              canExclude={canExclude}
              onPress={() => props.onLoader(loader === id ? null : id)}
              onExclude={() => props.onToggleExcludedLoader(id)}
            />
          ))}
        </Section>
      )}
      {hasGroups ? (
        facets?.categories.map((group) => (
          <CategorySection key={group.header} group={group} facets={facets} filters={filters} actions={props} />
        ))
      ) : (
        <Section title="Categories" disabled hint={loading ? "Loading…" : `Not offered by ${platformName}`} />
      )}
      {usesEnvironments(kind) && (
        <Section
          title="Environment"
          disabled={!facets?.environments}
          hint={loading ? "Loading…" : `Not offered by ${platformName}`}
        >
          {pinFirst(SIDES, (side) => filters.environments.includes(side.id)).map((side) => (
            <FilterRow
              key={side.id}
              label={side.label}
              picked={filters.environments.includes(side.id)}
              onPress={() => props.onToggleEnvironment(side.id)}
            />
          ))}
        </Section>
      )}
      <Section
        title="License"
        disabled={!facets?.open_source}
        hint={loading ? "Loading…" : `Not offered by ${platformName}`}
      >
        <FilterRow label="Open source" picked={filters.openSource} onPress={() => props.onOpenSource(!filters.openSource)} />
      </Section>
      {hasPicks && (
        <Button
          variant="ghost"
          onPress={props.onClear}
          className="w-full rounded-control border border-line bg-panel text-sm text-ink-1 hover:border-line-strong"
        >
          Reset filters
        </Button>
      )}
    </div>
  );
}

interface SectionProps {
  title: string;
  disabled?: boolean;
  hint?: string;
  children?: ReactNode;
}

function Section({ title, disabled = false, hint, children }: SectionProps) {
  return (
    <section
      aria-disabled={disabled || undefined}
      className={cn("space-y-1.5 rounded-panel border border-line bg-panel p-2 transition", disabled && "opacity-45")}
    >
      <h3 className="flex items-baseline justify-between gap-2 px-2 pt-1 font-mono text-[11px] tracking-wider text-ink-3 uppercase">
        {title}
        {disabled && hint && <span className="truncate text-[10px] normal-case tracking-normal text-ink-4">{hint}</span>}
      </h3>
      <SectionDisabled.Provider value={disabled}>
        <div className={cn("flex flex-col gap-0.5", disabled && "pointer-events-none")}>{children}</div>
      </SectionDisabled.Provider>
    </section>
  );
}

function VersionList({ selected, onSelect }: { selected: string | null; onSelect: (version: string | null) => void }) {
  const disabled = useContext(SectionDisabled);
  const [snapshots, setSnapshots] = useState(false);
  const { data: versions = [] } = useMinecraftVersions("vanilla", snapshots);
  const [search, setSearch] = useState("");
  const [expanded, setExpanded] = useState(false);
  const matching = pinFirst(
    versions.filter((version) => version.toLowerCase().includes(search.trim().toLowerCase())),
    (version) => version === selected,
  );
  const shown = expanded ? matching : matching.slice(0, SHOWN_VERSIONS);

  return (
    <>
      <SearchField aria-label="Search game versions" value={search} onChange={setSearch} isDisabled={disabled} className="mb-1 w-full">
        <SearchField.Group>
          <SearchField.SearchIcon />
          <SearchField.Input placeholder="Search versions" />
          <SearchField.ClearButton />
        </SearchField.Group>
      </SearchField>
      <CheckboxRow isSelected={snapshots} onChange={setSnapshots} isDisabled={disabled} className="px-2 py-1 text-xs text-ink-2">
        Show all versions
      </CheckboxRow>
      {shown.map((version) => (
        <FilterRow
          key={version}
          label={version}
          picked={selected === version}
          onPress={() => onSelect(selected === version ? null : version)}
        />
      ))}
      {matching.length > SHOWN_VERSIONS && (
        <ShowMore expanded={expanded} hidden={matching.length - SHOWN_VERSIONS} onPress={() => setExpanded(!expanded)} />
      )}
    </>
  );
}

interface CategorySectionProps {
  group: CategoryGroup;
  facets: Facets;
  filters: ContentFilters;
  actions: DiscoverSidebarProps;
}

function CategorySection({ group, facets, filters, actions }: CategorySectionProps) {
  const [expanded, setExpanded] = useState(false);
  const categories = pinFirst(
    group.categories,
    (category) => filters.categories.includes(category.id) || filters.excludedCategories.includes(category.id),
  );
  const shown = expanded ? categories : categories.slice(0, SHOWN_CATEGORIES);

  return (
    <Section title={titleize(group.header)}>
      {shown.map((category) => (
        <FilterRow
          key={category.id}
          label={titleize(category.name)}
          picked={filters.categories.includes(category.id)}
          excluded={filters.excludedCategories.includes(category.id)}
          canExclude={facets.exclusions}
          onPress={() => actions.onToggleCategory(category.id, facets.multiple)}
          onExclude={() => actions.onToggleExcluded(category.id)}
        />
      ))}
      {group.categories.length > SHOWN_CATEGORIES && (
        <ShowMore
          expanded={expanded}
          hidden={group.categories.length - SHOWN_CATEGORIES}
          onPress={() => setExpanded(!expanded)}
        />
      )}
    </Section>
  );
}

interface FilterRowProps {
  label: string;
  picked: boolean;
  excluded?: boolean;
  canExclude?: boolean;
  onPress: () => void;
  onExclude?: () => void;
}

function FilterRow({ label, picked, excluded = false, canExclude = false, onPress, onExclude }: FilterRowProps) {
  const disabled = useContext(SectionDisabled);
  return (
    <div className="group flex min-w-0 items-center gap-1 rounded-control">
      <Button
        variant="ghost"
        isDisabled={disabled}
        onPress={onPress}
        className={cn(
          "h-auto min-w-0 flex-1 justify-start rounded-control px-2 py-1.5 text-left text-[13px] font-normal",
          picked ? "bg-accent-tint text-ink-0" : excluded ? "text-ink-3 line-through" : "text-ink-2 hover:bg-raised",
        )}
      >
        <span className="truncate">{label}</span>
      </Button>
      <Button
        variant="ghost"
        isDisabled={disabled}
        aria-label={picked ? `Clear ${label}` : `Include ${label}`}
        aria-pressed={picked}
        onPress={onPress}
        className={cn(
          "size-6 min-w-0 shrink-0 rounded-md p-0",
          picked ? "bg-green/15 text-green" : "text-ink-4 hover:bg-raised hover:text-ink-1",
        )}
      >
        <Icon icon="lucide:plus" width={13} height={13} />
      </Button>
      {canExclude && onExclude && (
        <Button
          variant="ghost"
          isDisabled={disabled}
          aria-label={excluded ? `Stop excluding ${label}` : `Exclude ${label}`}
          aria-pressed={excluded}
          onPress={onExclude}
          className={cn(
            "size-6 min-w-0 shrink-0 rounded-md p-0",
            excluded ? "bg-rose/15 text-rose" : "text-ink-4 hover:bg-raised hover:text-ink-1",
          )}
        >
          <Icon icon="lucide:minus" width={13} height={13} />
        </Button>
      )}
    </div>
  );
}

function ShowMore({ expanded, hidden, onPress }: { expanded: boolean; hidden: number; onPress: () => void }) {
  const disabled = useContext(SectionDisabled);
  return (
    <Button
      variant="ghost"
      isDisabled={disabled}
      onPress={onPress}
      className="h-auto justify-center rounded-control px-2 py-1.5 text-xs text-ink-3 hover:bg-raised"
    >
      {expanded ? "Show fewer" : `Show ${hidden} more`}
    </Button>
  );
}

function titleize(text: string): string {
  return text
    .replace(/[-_]/g, " ")
    .split(" ")
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

function pinFirst<T>(items: readonly T[], isPinned: (item: T) => boolean): T[] {
  return [...items.filter(isPinned), ...items.filter((item) => !isPinned(item))];
}

function usesEnvironments(kind: ContentKind): boolean {
  return kind === "mod" || kind === "plugin" || kind === "modpack";
}
