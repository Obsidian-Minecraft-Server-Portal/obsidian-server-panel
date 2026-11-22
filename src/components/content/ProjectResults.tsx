import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { Project } from "../../api/content";
import { PLATFORM_NAMES, formatDownloads, projectKey } from "./meta";
import { ProjectBanner } from "./ProjectBanner";
import { ProjectIcon } from "./ProjectIcon";

export type { ViewMode as ResultView } from "../ui/ViewToggle";
import type { ViewMode as ResultView } from "../ui/ViewToggle";

interface ProjectResultsProps {
  projects: Project[];
  view: Exclude<ResultView, "table">;
  onOpen: (project: Project) => void;
}

interface ProjectItemProps {
  project: Project;
  onOpen: (project: Project) => void;
}

const META_CLASS = "truncate font-mono text-[11px] uppercase tracking-wider text-ink-3";

export function PlatformChip({ platform }: { platform: Project["platform"] }) {
  return (
    <span className="inline-flex items-center rounded-full border border-white/10 bg-black/40 px-2.5 py-1 font-mono text-[10px] tracking-widest text-ink-1 uppercase backdrop-blur">
      {PLATFORM_NAMES[platform]}
    </span>
  );
}

function ProjectCard({ project, onOpen }: ProjectItemProps) {
  return (
    <Button
      onPress={() => onOpen(project)}
      className="group h-full w-full min-w-0 flex-col items-stretch justify-start gap-0 overflow-hidden rounded-panel border border-line bg-panel p-0 text-left transition-colors duration-150 hover:border-line-strong hover:bg-raised/40"
    >
      <ProjectBanner src={project.banner_url} seed={projectKey(project)} className="h-36 shrink-0">
        <div className="absolute inset-0 bg-gradient-to-t from-panel via-transparent to-transparent" />
        <div className="absolute top-3 left-3">
          <PlatformChip platform={project.platform} />
        </div>
      </ProjectBanner>
      <div className="flex min-w-0 flex-1 flex-col gap-3 px-4 pt-3 pb-4">
        <div className="flex min-w-0 items-center gap-3">
          <ProjectIcon src={project.icon_url} alt="" className="size-9 shrink-0" />
          <p className="truncate font-semibold text-ink-0">{project.name}</p>
        </div>
        <p className="line-clamp-3 min-h-[3.9em] text-sm whitespace-normal text-ink-2">{project.summary}</p>
        <div className="mt-auto flex items-center justify-between gap-3 pt-1">
          <span className={cn(META_CLASS, "inline-flex items-center gap-1.5")}>
            <Icon icon="lucide:download" width={12} height={12} />
            {formatDownloads(project.downloads)}
          </span>
          <span className="rounded-full bg-raised px-3 py-1 text-xs font-medium text-ink-1 transition-colors group-hover:bg-accent group-hover:text-accent-ink">
            View
          </span>
        </div>
      </div>
    </Button>
  );
}

function ProjectRow({ project, onOpen }: ProjectItemProps) {
  return (
    <Button
      onPress={() => onOpen(project)}
      className="h-auto w-full min-w-0 items-center justify-start gap-4 rounded-control border border-line bg-panel px-4 py-3 text-left transition duration-200 hover:border-line-strong hover:bg-raised"
    >
      <ProjectIcon src={project.icon_url} alt="" className="size-9" />
      <div className="flex min-w-0 flex-1 flex-col items-start gap-0.5">
        <span className="max-w-full truncate font-medium text-ink-0">{project.name}</span>
        <span className="max-w-full truncate text-xs text-ink-3">{project.summary}</span>
      </div>
      <span className={cn(META_CLASS, "hidden sm:block")}>{PLATFORM_NAMES[project.platform]}</span>
      <span className={cn(META_CLASS, "w-20 text-right")}>{formatDownloads(project.downloads)}</span>
    </Button>
  );
}

export function ProjectResults({ projects, view, onOpen }: ProjectResultsProps) {
  if (view === "compact") {
    return (
      <div className="flex flex-col gap-2">
        {projects.map((project) => (
          <ProjectRow key={projectKey(project)} project={project} onOpen={onOpen} />
        ))}
      </div>
    );
  }
  return (
    <div className="grid auto-rows-fr gap-4 sm:grid-cols-2 2xl:grid-cols-3 min-[2200px]:grid-cols-4">
      {projects.map((project) => (
        <ProjectCard key={projectKey(project)} project={project} onOpen={onOpen} />
      ))}
    </div>
  );
}
