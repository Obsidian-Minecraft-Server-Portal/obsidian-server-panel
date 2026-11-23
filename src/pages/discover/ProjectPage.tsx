import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { Link, useParams, useSearchParams } from "react-router";
import { type PlatformId, type ProjectDetail, useInstalledContent, useProjectDetail, useQuickAdd } from "../../api/content";
import { hasPermission, useInstances } from "../../api/instances";
import { InstallTargetModal, type InstallAction } from "../../components/content/InstallTargetModal";
import { ModpackModal, type ModpackAction } from "../../components/content/ModpackModal";
import { acceptsLoader, parsePlatform, quickAddSummary } from "../../components/content/meta";
import { ProjectActions, type QuickAdd } from "../../components/content/project/ProjectActions";
import { ProjectChangelog } from "../../components/content/project/ProjectChangelog";
import { ProjectGallery } from "../../components/content/project/ProjectGallery";
import { ProjectHeader } from "../../components/content/project/ProjectHeader";
import { ProjectOverview } from "../../components/content/project/ProjectOverview";
import { ProjectVersions } from "../../components/content/project/ProjectVersions";
import { useInstalledProjects } from "../../components/content/useInstalledProjects";
import { EmptyState } from "../../components/ui/EmptyState";
import { SectionTabs, type SectionTab } from "../../components/ui/SectionTabs";

type Tab = "overview" | "gallery" | "changelog" | "versions";

const TABS: SectionTab<Tab>[] = [
  { id: "overview", label: "Overview" },
  { id: "gallery", label: "Gallery" },
  { id: "changelog", label: "Changelog" },
  { id: "versions", label: "Versions" },
];

export function ProjectPage() {
  const { platform: rawPlatform, projectId } = useParams();
  const platform = parsePlatform(rawPlatform);
  if (platform === null || !projectId) {
    return (
      <div className="px-4 py-6 sm:px-8">
        <EmptyState
          icon="lucide:package-x"
          title="Unknown project"
          description="This link does not point to a project on a supported platform."
        />
      </div>
    );
  }
  return <ProjectView platform={platform} projectId={projectId} />;
}

function ProjectView({ platform, projectId }: { platform: PlatformId; projectId: string }) {
  const detail = useProjectDetail(platform, projectId);
  if (detail.isError) {
    return (
      <div className="px-4 py-6 sm:px-8">
        <EmptyState icon="lucide:triangle-alert" title="Project unavailable" description={detail.error.message} />
      </div>
    );
  }
  if (!detail.data) return <p className="px-4 py-6 text-sm text-ink-3 sm:px-8">Loading project…</p>;
  return <ProjectBody detail={detail.data} />;
}

function ProjectBody({ detail }: { detail: ProjectDetail }) {
  const { project } = detail;
  const instances = useInstances();
  const [params, setParams] = useSearchParams();
  const [installAction, setInstallAction] = useState<InstallAction | null>(null);
  const [modpackAction, setModpackAction] = useState<ModpackAction | null>(null);

  const installTargets = (instances.data ?? []).filter(
    (entry) => hasPermission(entry, "content.install") && acceptsLoader(project.kind, entry.loader),
  );
  const target = installTargets.find((entry) => entry.id === Number.parseInt(params.get("instance") ?? "", 10)) ?? null;
  const quickAdd = useQuickAdd(target?.id ?? 0);
  const installed = useInstalledProjects(target?.id ?? null);
  const associateId = Number.parseInt(params.get("associate") ?? "", 10) || null;
  const installedContent = useInstalledContent(target?.id ?? 0, target !== null && associateId !== null, false);
  const associateItem = installedContent.data?.find((item) => item.id === associateId) ?? null;
  const association =
    target !== null && associateItem !== null
      ? { instanceId: target.id, contentId: associateItem.id, filename: associateItem.filename }
      : null;
  const tab = TABS.find((entry) => entry.id === params.get("tab"))?.id ?? "overview";

  const update = (patch: Record<string, string | null>) =>
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        for (const [key, value] of Object.entries(patch)) {
          if (value === null) next.delete(key);
          else next.set(key, value);
        }
        return next;
      },
      { replace: true },
    );

  const back = new URLSearchParams({ kind: project.kind, platform: String(project.platform) });
  if (target) back.set("instance", String(target.id));
  if (associateId !== null) back.set("associate", String(associateId));

  return (
    <div className="space-y-6 px-4 py-6 sm:px-8">
      <Link
        to={`/discover?${back}`}
        className="inline-flex items-center gap-2 text-sm text-ink-2 transition-colors hover:text-ink-0"
      >
        <Icon icon="lucide:arrow-left" width={14} height={14} />
        Back to Discover
      </Link>
      <ProjectHeader detail={detail}>
        <ProjectActions
          project={project}
          webUrl={detail.web_url}
          installTargets={installTargets}
          isLoading={instances.isPending}
          target={target}
          isInstalled={installed.has(project.platform, project.id)}
          onTargetChange={(id) => update({ instance: id === null ? null : String(id) })}
          quickAdd={quickAdd}
          onChooseVersion={() => update({ tab: "versions" })}
        />
      </ProjectHeader>
      <div className="space-y-5">
        <SectionTabs
          label="Project sections"
          tabs={TABS}
          value={tab}
          onChange={(next) => update({ tab: next })}
          className="max-w-full overflow-x-auto"
        />
        <QuickAddNotice quickAdd={quickAdd} instanceName={target?.name ?? "the instance"} />
        {tab === "overview" && <ProjectOverview detail={detail} />}
        {tab === "gallery" && <ProjectGallery name={project.name} images={detail.gallery} />}
        {tab === "changelog" && <ProjectChangelog platform={project.platform} projectId={project.id} />}
        {tab === "versions" && (
          <ProjectVersions
            project={project}
            gameVersions={detail.game_versions}
            target={target}
            quickAdd={quickAdd}
            isInstalledVersion={(versionId) => installed.hasVersion(project.platform, project.id, versionId)}
            association={association}
            onModpack={setModpackAction}
            onInstall={setInstallAction}
          />
        )}
      </div>
      <InstallTargetModal
        action={installAction}
        defaultInstanceId={target?.id ?? null}
        onClose={() => setInstallAction(null)}
      />
      <ModpackModal action={modpackAction} onClose={() => setModpackAction(null)} />
    </div>
  );
}

function QuickAddNotice({ quickAdd, instanceName }: { quickAdd: QuickAdd; instanceName: string }) {
  if (quickAdd.error) return <p className="text-sm text-rose">{quickAdd.error.message}</p>;
  if (!quickAdd.isSuccess) return null;
  return (
    <p className="flex items-center gap-2 text-sm text-green">
      <Icon icon="lucide:circle-check" width={16} height={16} />
      {quickAddSummary(quickAdd.data, instanceName)}
    </p>
  );
}
