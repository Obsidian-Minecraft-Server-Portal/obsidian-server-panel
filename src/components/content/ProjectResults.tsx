import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { ReactNode } from "react";
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
  renderActions: (project: Project) => ReactNode;
}

interface ProjectItemProps {
  project: Project;
  onOpen: (project: Project) => void;
  actions: ReactNode;
}

const META_CLASS = "truncate font-mono text-[11px] uppercase tracking-wider text-ink-3";

export function PlatformChip({ platform }: { platform: Project["platform"] }) {
  return (
    <span className="inline-flex items-center rounded-full border border-white/10 bg-black/40 px-2.5 py-1 font-mono text-[10px] tracking-widest text-ink-1 uppercase backdrop-blur">
      {PLATFORM_NAMES[platform]}
    </span>
  );
}

function ProjectCard({ project, onOpen, actions }: ProjectItemProps) {
  return (
    <article className="group flex h-full min-w-0 flex-col overflow-hidden rounded-panel border border-line bg-panel transition-colors duration-150 hover:border-line-strong hover:bg-raised/40">
      <button type="button" onClick={() => onOpen(project)} className="block text-left">
        <ProjectBanner src={project.banner_url} seed={projectKey(project)} className="h-36 shrink-0">
          <div className="absolute inset-0 bg-gradient-to-t from-panel via-transparent to-transparent" />
          <div className="absolute top-3 left-3">
            <PlatformChip platform={project.platform} />
          </div>
        </ProjectBanner>
      </button>
      <div className="flex min-w-0 flex-1 flex-col gap-3 px-4 pt-3 pb-4">
        <div className="flex min-w-0 items-center gap-3">
          <ProjectIcon src={project.icon_url} alt="" className="size-9 shrink-0" />
          <button
            type="button"
            onClick={() => onOpen(project)}
            className="min-w-0 truncate text-left font-semibold text-ink-0 hover:underline"
          >
            {project.name}
          </button>
        </div>
        <p className="line-clamp-3 min-h-[3.9em] text-sm text-ink-2">{project.summary}</p>
        <div className="mt-auto flex flex-wrap items-center justify-between gap-3 pt-1">
          <span className={cn(META_CLASS, "inline-flex items-center gap-1.5")}>
            <Icon icon="lucide:download" width={12} height={12} />
            {formatDownloads(project.downloads)}
          </span>
          <div className="flex flex-wrap gap-2">{actions}</div>
        </div>
      </div>
    </article>
  );
}

function ProjectRow({ project, onOpen, actions }: ProjectItemProps) {
  return (
    <div className="flex w-full min-w-0 items-center gap-4 rounded-control border border-line bg-panel px-4 py-3 transition duration-200 hover:border-line-strong hover:bg-raised">
      <ProjectIcon src={project.icon_url} alt="" className="size-9 shrink-0" />
      <button type="button" onClick={() => onOpen(project)} className="flex min-w-0 flex-1 flex-col items-start gap-0.5 text-left">
        <span className="max-w-full truncate font-medium text-ink-0">{project.name}</span>
        <span className="max-w-full truncate text-xs text-ink-3">{project.summary}</span>
      </button>
      <span className={cn(META_CLASS, "hidden sm:block")}>{PLATFORM_NAMES[project.platform]}</span>
      <span className={cn(META_CLASS, "w-20 text-right")}>{formatDownloads(project.downloads)}</span>
      <div className="flex shrink-0 gap-2">{actions}</div>
    </div>
  );
}

export function ProjectResults({ projects, view, onOpen, renderActions }: ProjectResultsProps) {
  if (view === "compact") {
    return (
      <div className="flex flex-col gap-2">
        {projects.map((project) => (
          <ProjectRow key={projectKey(project)} project={project} onOpen={onOpen} actions={renderActions(project)} />
        ))}
      </div>
    );
  }
  return (
    <div className="grid auto-rows-fr gap-4 sm:grid-cols-2 2xl:grid-cols-3 min-[2200px]:grid-cols-4">
      {projects.map((project) => (
        <ProjectCard key={projectKey(project)} project={project} onOpen={onOpen} actions={renderActions(project)} />
      ))}
    </div>
  );
}
