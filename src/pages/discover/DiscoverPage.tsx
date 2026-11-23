import { Icon } from "@iconify-icon/react";
import { type ReactNode, useMemo } from "react";
import { Link, useNavigate } from "react-router";
import type { Project, SideFilter } from "../../api/content";
import { useContentFacets, useContentSearch, usePlatforms, useQuickAdd } from "../../api/content";
import { hasPermission, useInstance } from "../../api/instances";
import { DiscoverTabs, SourceChips } from "../../components/content/DiscoverFilters";
import { DiscoverHero } from "../../components/content/DiscoverHero";
import { DiscoverSidebar } from "../../components/content/DiscoverSidebar";
import { KIND_LABELS, LOADER_BOUND_KINDS, PLATFORM_NAMES, acceptsLoader } from "../../components/content/meta";
import { ProjectResults } from "../../components/content/ProjectResults";
import { ProjectTable } from "../../components/content/ProjectTable";
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
  const browsing = discover.debouncedQuery.trim() === "" && projects.length > 0;
  const hero = browsing ? projects[0] : null;
  const listed = hero ? projects.slice(1) : projects;

  const quickAddTarget = instance.data && hasPermission(instance.data, "content.install") ? instance.data : null;
  const quickAddable = (project: Project) =>
    quickAddTarget !== null &&
    LOADER_BOUND_KINDS.has(project.kind) &&
    acceptsLoader(project.kind, quickAddTarget.loader);

  const openProject = (project: Project) => {
    const query = discover.instanceId === null ? "" : `?instance=${discover.instanceId}`;
    navigate(`/discover/projects/${project.platform}/${encodeURIComponent(project.id)}${query}`);
  };

  const renderActions = (project: Project) => (
    <>
      {quickAddable(project) && (
        <PrimaryButton
          size="sm"
          isDisabled={quickAdd.isPending}
          onPress={() => quickAdd.mutate({ platform: project.platform, projectId: project.id })}
        >
          <Icon icon="lucide:zap" width={14} height={14} />
          Quick add
        </PrimaryButton>
      )}
      <GhostButton size="sm" onPress={() => openProject(project)}>
        View more
      </GhostButton>
    </>
  );

  return (
    <>
      <PageHeader
        title="Discover"
        subtitle={`Search ${platformName} for ${KIND_LABELS[discover.kind].toLowerCase()}`}
        className="mb-0 border-b-0 pb-4"
        search={{ value: discover.query, onChange: discover.setQuery, placeholder: "Search content" }}
        actions={<ViewToggle label="Result view" value={discover.view} onChange={discover.setView} />}
      />
      <div className="space-y-5 px-8 pb-8">
        <div className="space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <SourceChips
              kind={discover.kind}
              platform={discover.platform}
              platforms={platforms}
              onPlatform={discover.setPlatform}
            />
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
        <div className="flex items-start gap-6">
          <DiscoverSidebar
            kind={discover.kind}
            platformName={platformName}
            facets={facets}
            filters={filters}
            loader={discover.loader}
            mcVersion={discover.mcVersion}
            onLoader={discover.setLoader}
            onMcVersion={discover.setMcVersion}
            onToggleCategory={discover.toggleCategory}
            onToggleExcluded={discover.toggleExcluded}
            onToggleEnvironment={(side) => discover.toggleEnvironment(side, environments)}
            onOpenSource={discover.setOpenSource}
            onClear={discover.clearFilters}
          />
          <div className="min-w-0 flex-1 space-y-5">
            {quickAdd.isSuccess && (
              <p className="flex items-center gap-2 text-sm text-green">
                <Icon icon="lucide:circle-check" width={16} height={16} />
                Added {quickAdd.data.length} {quickAdd.data.length === 1 ? "file" : "files"} to {instance.data?.name ?? "the instance"}.
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
              total={search.data?.pages[0]?.total ?? 0}
              onOpen={openProject}
              renderActions={renderActions}
            />
            {search.hasNextPage && (
              <div className="flex justify-center">
                <GhostButton isDisabled={search.isFetchingNextPage} onPress={() => void search.fetchNextPage()}>
                  {search.isFetchingNextPage ? "Loading…" : "Load more"}
                </GhostButton>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
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
