import { Button, cn } from "@heroui/react";
import type { Project } from "../../api/content";
import { PLATFORM_NAMES, formatDownloads, projectKey } from "./meta";
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

function ProjectCard({ project, onOpen }: ProjectItemProps) {
  return (
    <Button
      onPress={() => onOpen(project)}
      className="h-auto flex-col items-stretch justify-start gap-4 rounded-panel border border-line bg-panel p-4 text-left transition duration-200 hover:-translate-y-px hover:border-accent/40 hover:shadow-glow"
    >
      <div className="flex min-w-0 gap-3">
        <ProjectIcon src={project.icon_url} alt="" className="size-14" />
        <div className="min-w-0 space-y-1">
          <p className="truncate font-semibold text-ink-0">{project.name}</p>
          <p className={META_CLASS}>
            {PLATFORM_NAMES[project.platform]} · {formatDownloads(project.downloads)} downloads
          </p>
        </div>
      </div>
      <p className="line-clamp-2 text-sm text-ink-3">{project.summary}</p>
    </Button>
  );
}

function ProjectRow({ project, onOpen }: ProjectItemProps) {
  return (
    <Button
      onPress={() => onOpen(project)}
      className="h-auto w-full items-center justify-start gap-4 rounded-control border border-line bg-panel px-4 py-3 text-left transition duration-200 hover:border-accent/40 hover:bg-raised"
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
    <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
      {projects.map((project) => (
        <ProjectCard key={projectKey(project)} project={project} onOpen={onOpen} />
      ))}
    </div>
  );
}
