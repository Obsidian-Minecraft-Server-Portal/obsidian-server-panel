import { Drawer } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { ContentKind, Project, ProjectVersion, VersionFilter } from "../../api/content";
import { useProjectVersions } from "../../api/content";
import { PLATFORM_NAMES, formatDownloads } from "./meta";
import { ProjectIcon } from "./ProjectIcon";
import { VersionRow } from "./VersionRow";

interface ProjectDrawerProps {
  project: Project | null;
  kind: ContentKind;
  filter: VersionFilter;
  onClose: () => void;
  onAction: (version: ProjectVersion) => void;
}

export function ProjectDrawer({ project, kind, filter, onClose, onAction }: ProjectDrawerProps) {
  return (
    <Drawer.Backdrop isOpen={project !== null} onOpenChange={(open) => !open && onClose()}>
      <Drawer.Content placement="right">
        <Drawer.Dialog className="w-full max-w-xl bg-panel text-ink-1">
          {project && (
            <>
              <Drawer.Header className="gap-4 border-b border-line">
                <ProjectIcon src={project.icon_url} alt="" className="size-16" />
                <div className="min-w-0 flex-1 space-y-1">
                  <Drawer.Heading className="truncate text-xl font-semibold text-ink-0">{project.name}</Drawer.Heading>
                  <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">
                    {PLATFORM_NAMES[project.platform]} · {formatDownloads(project.downloads)} downloads
                  </p>
                </div>
                <Drawer.CloseTrigger aria-label="Close" />
              </Drawer.Header>
              <Drawer.Body className="space-y-5">
                <p className="text-sm text-ink-2">{project.summary}</p>
                <VersionList project={project} kind={kind} filter={filter} onAction={onAction} />
              </Drawer.Body>
            </>
          )}
        </Drawer.Dialog>
      </Drawer.Content>
    </Drawer.Backdrop>
  );
}

interface VersionListProps {
  project: Project;
  kind: ContentKind;
  filter: VersionFilter;
  onAction: (version: ProjectVersion) => void;
}

function VersionList({ project, kind, filter, onAction }: VersionListProps) {
  const { data, isLoading, isError, error } = useProjectVersions(project.platform, project.id, filter);

  if (isLoading) return <p className="text-sm text-ink-3">Loading versions…</p>;
  if (isError) return <p className="text-sm text-rose">{error.message}</p>;
  if (!data?.length) {
    return (
      <div className="flex items-center gap-2 text-sm text-ink-3">
        <Icon icon="lucide:circle-off" width={16} height={16} />
        No versions match the current filters.
      </div>
    );
  }

  return (
    <section className="space-y-2.5">
      <h3 className="font-mono text-[11px] font-medium tracking-wider text-ink-3 uppercase">Versions</h3>
      {data.map((version) => (
        <VersionRow key={version.id} version={version} kind={kind} onAction={onAction} />
      ))}
    </section>
  );
}
