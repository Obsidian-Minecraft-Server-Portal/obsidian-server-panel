import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import type { Project } from "../../api/content";
import { PlatformChip } from "./ProjectResults";
import { formatDownloads, projectKey } from "./meta";
import { ProjectBanner } from "./ProjectBanner";

interface DiscoverHeroProps {
  project: Project;
  onOpen: (project: Project) => void;
}

export function DiscoverHero({ project, onOpen }: DiscoverHeroProps) {
  return (
    <section className="group relative isolate h-64 overflow-hidden rounded-[18px] border border-line">
      <ProjectBanner src={project.banner_url} seed={projectKey(project)} className="absolute inset-0">
        <div className="absolute inset-0 bg-gradient-to-t from-canvas via-canvas/50 to-black/10" />
      </ProjectBanner>
      <div className="relative flex h-full flex-col justify-end gap-3 p-7">
        <div className="flex items-center gap-3">
          <PlatformChip platform={project.platform} />
          <span className="font-mono text-[11px] tracking-wider text-accent uppercase">Most downloaded</span>
        </div>
        <h2 className="max-w-3xl truncate text-3xl font-extrabold text-ink-0">{project.name}</h2>
        <p className="line-clamp-2 max-w-2xl text-sm text-ink-2">{project.summary}</p>
        <div className="flex flex-wrap items-center gap-4">
          <Button
            onPress={() => onOpen(project)}
            className="gap-2 rounded-control bg-gradient-to-b from-accent to-accent-dim px-5 py-2.5 text-sm font-semibold text-accent-ink shadow-[inset_0_1px_0_rgba(255,255,255,0.25)] transition hover:-translate-y-px hover:shadow-glow"
          >
            <Icon icon="lucide:book-open" width={14} height={14} />
            View
          </Button>
          <span className="inline-flex items-center gap-1.5 font-mono text-xs text-ink-3">
            <Icon icon="lucide:download" width={12} height={12} />
            {formatDownloads(project.downloads)} downloads
          </span>
        </div>
      </div>
    </section>
  );
}
