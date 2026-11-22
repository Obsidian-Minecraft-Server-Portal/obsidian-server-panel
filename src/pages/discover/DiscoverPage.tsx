import { Icon } from "@iconify-icon/react";
import { useMemo, useState } from "react";
import { Link } from "react-router";
import type { Project, ProjectVersion } from "../../api/content";
import { useContentSearch, usePlatforms } from "../../api/content";
import { useInstance } from "../../api/instances";
import { DiscoverSelects, DiscoverTabs } from "../../components/content/DiscoverFilters";
import { ViewToggle } from "../../components/ui/ViewToggle";
import { InstallTargetModal, type InstallAction } from "../../components/content/InstallTargetModal";
import { ModpackModal, type ModpackAction } from "../../components/content/ModpackModal";
import { ProjectDrawer } from "../../components/content/ProjectDrawer";
import { ProjectResults } from "../../components/content/ProjectResults";
import { ProjectTable } from "../../components/content/ProjectTable";
import { EmptyState } from "../../components/ui/EmptyState";
import { GhostButton } from "../../components/ui/GhostButton";
import { PageHeader } from "../../components/ui/PageHeader";
import { useDiscoverState } from "./useDiscoverState";

export function DiscoverPage() {
  const discover = useDiscoverState();
  const { data: platforms = [] } = usePlatforms();
  const instance = useInstance(discover.instanceId ?? Number.NaN);
  const [drawerProject, setDrawerProject] = useState<Project | null>(null);
  const [installAction, setInstallAction] = useState<InstallAction | null>(null);
  const [modpackAction, setModpackAction] = useState<ModpackAction | null>(null);

  const isModpack = discover.kind === "modpack";
  const loader = isModpack ? null : discover.loader;
  const mcVersion = isModpack ? null : discover.mcVersion;

  const search = useContentSearch({
    platform: discover.platform,
    kind: discover.kind,
    query: discover.debouncedQuery,
    loader,
    mcVersion,
  });
  const projects = useMemo(() => search.data?.pages.flatMap((page) => page.items) ?? [], [search.data]);

  const chooseVersion = (version: ProjectVersion) => {
    if (!drawerProject) return;
    if (isModpack) setModpackAction({ project: drawerProject, version });
    else setInstallAction({ project: drawerProject, version });
  };

  return (
    <>
      <PageHeader
        title="Discover"
        subtitle="Modpacks, mods and plugins from Modrinth, CurseForge, Hangar and Spiget."
        search={{ value: discover.query, onChange: discover.setQuery, placeholder: "Search content" }}
        actions={<ViewToggle label="Result view" value={discover.view} onChange={discover.setView} />}
      />
      <div className="space-y-6 px-8 pb-8">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <DiscoverTabs kind={discover.kind} onKind={discover.setKind} />
          {instance.data && (
            <Link
              to={`/instances/${instance.data.id}/content`}
              className="inline-flex items-center gap-2 rounded-full bg-accent-tint px-3 py-1.5 font-mono text-[11px] tracking-wider text-accent uppercase"
            >
              <Icon icon="lucide:crosshair" width={14} height={14} />
              For {instance.data.name}
            </Link>
          )}
        </div>
        <DiscoverSelects
          kind={discover.kind}
          platform={discover.platform}
          platforms={platforms}
          onPlatform={discover.setPlatform}
          loader={discover.loader}
          onLoader={discover.setLoader}
          mcVersion={discover.mcVersion}
          onMcVersion={discover.setMcVersion}
        />
        <Results
          projects={projects}
          view={discover.view}
          isPending={search.isPending}
          isError={search.isError}
          errorMessage={search.error?.message}
          total={search.data?.pages[0]?.total ?? 0}
          onOpen={setDrawerProject}
        />
        {search.hasNextPage && (
          <div className="flex justify-center">
            <GhostButton isDisabled={search.isFetchingNextPage} onPress={() => void search.fetchNextPage()}>
              {search.isFetchingNextPage ? "Loading…" : "Load more"}
            </GhostButton>
          </div>
        )}
      </div>
      <ProjectDrawer
        project={drawerProject}
        kind={discover.kind}
        filter={{ loader, mcVersion }}
        onClose={() => setDrawerProject(null)}
        onAction={chooseVersion}
      />
      <InstallTargetModal
        action={installAction}
        defaultInstanceId={discover.instanceId}
        onClose={() => setInstallAction(null)}
      />
      <ModpackModal action={modpackAction} onClose={() => setModpackAction(null)} />
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
}

function Results({ projects, view, isPending, isError, errorMessage, total, onOpen }: ResultsProps) {
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
        <ProjectTable projects={projects} onOpen={onOpen} />
      ) : (
        <ProjectResults projects={projects} view={view} onOpen={onOpen} />
      )}
    </div>
  );
}
