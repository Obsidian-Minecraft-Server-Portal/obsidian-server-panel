import { Drawer } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { type ReactNode, useMemo, useRef, useState } from "react";
import { Link, useNavigate } from "react-router";
import type { Project, SideFilter } from "../../api/content";
import { useContentFacets, useContentSearch, usePlatforms, useQuickAdd } from "../../api/content";
import { hasPermission, useInstance } from "../../api/instances";
import { DiscoverTabs, SourceChips } from "../../components/content/DiscoverFilters";
import { DiscoverHero } from "../../components/content/DiscoverHero";
import { DiscoverSidebar } from "../../components/content/DiscoverSidebar";
import { KIND_LABELS, PLATFORM_NAMES, QUICK_ADD_KINDS, acceptsLoader, quickAddSummary } from "../../components/content/meta";
import { ProjectResults } from "../../components/content/ProjectResults";
import { ProjectTable } from "../../components/content/ProjectTable";
import { useInstalledProjects } from "../../components/content/useInstalledProjects";
import { useLoadMoreOnView } from "../../components/content/useLoadMoreOnView";
import { EmptyState } from "../../components/ui/EmptyState";
import { GhostButton } from "../../components/ui/GhostButton";
import { PageHeader } from "../../components/ui/PageHeader";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { ViewToggle } from "../../components/ui/ViewToggle";
import { useDiscoverState } from "./useDiscoverState";

export function DiscoverPage() {
  const navigate = useNavigate();
  const { data: platforms = [] } = usePlatforms();
  const discover = useDiscoverState(platforms);
  const instance = useInstance(discover.instanceId ?? Number.NaN);
  const facets = useContentFacets(discover.platform, discover.kind).data;
  const quickAdd = useQuickAdd(discover.instanceId ?? 0);
  const installed = useInstalledProjects(discover.instanceId);
  const resultsRef = useRef<HTMLDivElement>(null);
  const [filtersOpen, setFiltersOpen] = useState(false);

  const isModpack = discover.kind === "modpack";
  const loader = isModpack ? null : discover.loader;
  const mcVersion = discover.mcVersion;
  const platformName = PLATFORM_NAMES[discover.platform];
  const environments: SideFilter[] = discover.environmentsChosen
    ? discover.filters.environments
    : facets?.environments && discover.instanceId !== null
      ? ["server"]
      : [];
  const filters = { ...discover.filters, environments };

  const search = useContentSearch({
    platform: discover.platform,
    kind: discover.kind,
    query: discover.debouncedQuery,
    loader,
    mcVersion,
    filters,
  });
  const projects = useMemo(() => search.data?.pages.flatMap((page) => page.items) ?? [], [search.data]);
  const total = search.data?.pages[0]?.total ?? 0;
  const browsing = discover.debouncedQuery.trim() === "" && projects.length > 0;
  const hero = browsing ? projects[0] : null;
  const listed = hero ? projects.slice(1) : projects;
  const loadNext = () => void search.fetchNextPage();
  const sentinel = useLoadMoreOnView(resultsRef, search.hasNextPage && !search.isFetchingNextPage, loadNext);

  const quickAddTarget = instance.data && hasPermission(instance.data, "content.install") ? instance.data : null;
  const quickAddable = (project: Project) =>
    quickAddTarget !== null &&
    QUICK_ADD_KINDS.has(project.kind) &&
    acceptsLoader(project.kind, quickAddTarget.loader);

  const openProject = (project: Project) => {
    const query = discover.instanceId === null ? "" : `?instance=${discover.instanceId}`;
    navigate(`/discover/projects/${project.platform}/${encodeURIComponent(project.id)}${query}`);
  };

  const renderActions = (project: Project) => (
    <>
      {installed.has(project.platform, project.id) ? (
        <GhostButton size="sm" isDisabled>
          <Icon icon="lucide:check" width={14} height={14} />
          Installed
        </GhostButton>
      ) : (
        quickAddable(project) && (
          <PrimaryButton
            size="sm"
            isDisabled={quickAdd.isPending}
            onPress={() => quickAdd.mutate({ platform: project.platform, projectId: project.id, kind: project.kind })}
          >
            <Icon icon="lucide:zap" width={14} height={14} />
            Quick add
          </PrimaryButton>
        )
      )}
      <GhostButton size="sm" onPress={() => openProject(project)}>
        View more
      </GhostButton>
    </>
  );

  const sidebar = (
    <DiscoverSidebar
      kind={discover.kind}
      platformName={platformName}
      facets={facets}
      filters={filters}
      loader={discover.loader}
      mcVersion={discover.mcVersion}
      onLoader={discover.setLoader}
      onToggleExcludedLoader={discover.toggleExcludedLoader}
      onMcVersion={discover.setMcVersion}
      onToggleCategory={discover.toggleCategory}
      onToggleExcluded={discover.toggleExcluded}
      onToggleEnvironment={(side) => discover.toggleEnvironment(side, environments)}
      onOpenSource={discover.setOpenSource}
      onClear={discover.clearFilters}
    />
  );

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="shrink-0 space-y-4 border-b border-line px-4 pt-5 pb-4 sm:px-8">
        <PageHeader
          title="Discover"
          subtitle={`Search ${platformName} for ${KIND_LABELS[discover.kind].toLowerCase()}`}
          className="mb-0 border-b-0 px-0 pt-0 pb-0"
          search={{ value: discover.query, onChange: discover.setQuery, placeholder: "Search content" }}
          actions={<ViewToggle label="Result view" value={discover.view} onChange={discover.setView} />}
        />
        <div className="flex flex-wrap items-center justify-between gap-4">
          <SourceChips kind={discover.kind} platform={discover.platform} platforms={platforms} onPlatform={discover.setPlatform} />
          {instance.data && (
            <Link
              to={`/instances/${instance.data.id}/content`}
              className="inline-flex items-center gap-2 rounded-full bg-secondary-tint px-3 py-1.5 font-mono text-[11px] tracking-wider text-secondary-soft uppercase"
            >
              <Icon icon="lucide:crosshair" width={14} height={14} />
              For {instance.data.name}
            </Link>
          )}
        </div>
        <DiscoverTabs kind={discover.kind} platform={discover.platform} platforms={platforms} onKind={discover.setKind} />
      </div>
      <div className="flex min-h-0 flex-1 gap-6 px-4 sm:px-8">
        <div className="hidden w-60 shrink-0 overflow-y-auto py-5 lg:block">{sidebar}</div>
        <div ref={resultsRef} className="min-h-0 min-w-0 flex-1 overflow-y-auto py-5">
          <div className="space-y-5 pb-8">
            <div className="lg:hidden">
              <GhostButton onPress={() => setFiltersOpen(true)}>
                <Icon icon="lucide:sliders-horizontal" width={15} height={15} />
                Filters
              </GhostButton>
            </div>
            {quickAdd.isSuccess && (
              <p className="flex items-center gap-2 text-sm text-green">
                <Icon icon="lucide:circle-check" width={16} height={16} />
                {quickAddSummary(quickAdd.data, instance.data?.name ?? "the instance")}
              </p>
            )}
            {quickAdd.error && <p className="text-sm text-rose">{quickAdd.error.message}</p>}
            {hero && <DiscoverHero project={hero} onOpen={openProject} />}
            <Results
              projects={listed}
              view={discover.view}
              isPending={search.isPending}
              isError={search.isError}
              errorMessage={search.error?.message}
              total={total}
              onOpen={openProject}
              renderActions={renderActions}
            />
            <div ref={sentinel} className="flex justify-center font-mono text-[11px] tracking-wider text-ink-3 uppercase">
              {search.isFetchingNextPage
                ? "Loading more…"
                : !search.hasNextPage && projects.length > 0
                  ? `All ${total.toLocaleString()} results loaded`
                  : null}
            </div>
          </div>
        </div>
      </div>
      <Drawer.Backdrop isOpen={filtersOpen} onOpenChange={setFiltersOpen}>
        <Drawer.Content placement="left">
          <Drawer.Dialog>
            <Drawer.Header>
              <Drawer.Heading>Filters</Drawer.Heading>
              <Drawer.CloseTrigger />
            </Drawer.Header>
            <Drawer.Body>{sidebar}</Drawer.Body>
          </Drawer.Dialog>
        </Drawer.Content>
      </Drawer.Backdrop>
    </div>
  );
}

interface ResultsProps {
  projects: Project[];
  view: "grid" | "compact" | "table";
  isPending: boolean;
  isError: boolean;
  errorMessage: string | undefined;
  total: number;
  onOpen: (project: Project) => void;
  renderActions: (project: Project) => ReactNode;
}

function Results({ projects, view, isPending, isError, errorMessage, total, onOpen, renderActions }: ResultsProps) {
  if (isError) return <p className="text-sm text-rose">{errorMessage ?? "Search failed."}</p>;
  if (isPending) return <p className="font-mono text-xs text-ink-3">Searching…</p>;
  if (projects.length === 0) {
    return <EmptyState icon="lucide:search-x" title="Nothing found" description="Try another search or relax the filters." />;
  }
  return (
    <div className="space-y-4">
      <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">
        {total.toLocaleString()} results · showing {projects.length}
      </p>
      {view === "table" ? (
        <ProjectTable projects={projects} onOpen={onOpen} renderActions={renderActions} />
      ) : (
        <ProjectResults projects={projects} view={view} onOpen={onOpen} renderActions={renderActions} />
      )}
    </div>
  );
}
