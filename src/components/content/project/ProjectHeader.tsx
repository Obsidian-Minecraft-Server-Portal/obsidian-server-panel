import { Icon } from "@iconify-icon/react";
import { format } from "date-fns";
import type { ReactNode } from "react";
import type { ProjectDetail } from "../../../api/content";
import { Chip } from "../../ui/Chip";
import { KIND_LABELS, formatDownloads, projectKey } from "../meta";
import { PlatformChip } from "../ProjectResults";
import { ProjectBanner } from "../ProjectBanner";
import { ProjectIcon } from "../ProjectIcon";

interface ProjectHeaderProps {
  detail: ProjectDetail;
  children?: ReactNode;
}

export function ProjectHeader({ detail, children }: ProjectHeaderProps) {
  const { project } = detail;
  return (
    <section className="overflow-hidden rounded-hero border border-line bg-panel">
      <ProjectBanner src={project.banner_url} seed={projectKey(project)} className="h-48">
        <div className="absolute inset-0 bg-gradient-to-t from-panel via-panel/40 to-transparent" />
        <div className="absolute top-4 left-5">
          <PlatformChip platform={project.platform} />
        </div>
      </ProjectBanner>
      <div className="relative flex flex-col gap-6 px-5 pb-6 sm:px-7 sm:pb-7 lg:flex-row lg:items-end lg:justify-between">
        <div className="flex min-w-0 flex-col gap-4 sm:flex-row sm:items-start sm:gap-5">
          <ProjectIcon src={project.icon_url} alt="" className="-mt-12 size-24 shrink-0 border-4 border-panel" />
          <div className="min-w-0 space-y-2.5 sm:pt-3">
            <p className="font-mono text-[11px] tracking-wider text-secondary uppercase">{KIND_LABELS[project.kind]}</p>
            <h1 className="text-2xl leading-tight font-extrabold break-words text-ink-0 sm:text-3xl">{project.name}</h1>
            {detail.authors.length > 0 && <p className="text-sm text-ink-3">by {detail.authors.join(", ")}</p>}
            <p className="max-w-3xl text-sm text-ink-2">{project.summary}</p>
            <Stats detail={detail} />
          </div>
        </div>
        {children && <div className="shrink-0">{children}</div>}
      </div>
    </section>
  );
}

function Stats({ detail }: { detail: ProjectDetail }) {
  return (
    <div className="flex flex-wrap gap-2 [&>span]:whitespace-nowrap">
      <Chip>
        <Icon icon="lucide:download" width={12} height={12} />
        {formatDownloads(detail.project.downloads)} downloads
      </Chip>
      {detail.followers !== null && (
        <Chip>
          <Icon icon="lucide:heart" width={12} height={12} />
          {formatDownloads(detail.followers)} followers
        </Chip>
      )}
      {detail.license && (
        <Chip>
          <Icon icon="lucide:scale" width={12} height={12} />
          {detail.license}
        </Chip>
      )}
      {detail.updated_at && (
        <Chip>
          <Icon icon="lucide:clock" width={12} height={12} />
          Updated {format(new Date(detail.updated_at), "d MMM yyyy")}
        </Chip>
      )}
    </div>
  );
}
