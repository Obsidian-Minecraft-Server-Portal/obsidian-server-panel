import { Button } from "@heroui/react";
import type { Project } from "../../api/content";
import { PLATFORM_NAMES, formatDownloads, projectKey } from "./meta";

interface ProjectTableProps {
  projects: Project[];
  onOpen: (project: Project) => void;
}

const HEAD_CLASS = "px-4 py-3 text-left font-mono text-[11px] font-medium tracking-wider text-ink-3 uppercase";

export function ProjectTable({ projects, onOpen }: ProjectTableProps) {
  return (
    <div className="overflow-x-auto rounded-panel border border-line bg-panel">
      <table className="w-full min-w-[36rem] border-collapse text-sm">
        <thead className="border-b border-line">
          <tr>
            <th scope="col" className={HEAD_CLASS}>Name</th>
            <th scope="col" className={HEAD_CLASS}>Platform</th>
            <th scope="col" className={`${HEAD_CLASS} text-right`}>Downloads</th>
          </tr>
        </thead>
        <tbody>
          {projects.map((project) => (
            <tr key={projectKey(project)} className="border-b border-line last:border-0 hover:bg-raised/60">
              <td className="px-4 py-3">
                <Button
                  variant="ghost"
                  onPress={() => onOpen(project)}
                  className="h-auto min-w-0 justify-start p-0 text-left font-medium text-ink-0 hover:text-accent"
                >
                  {project.name}
                </Button>
                <p className="max-w-xl truncate text-xs text-ink-3">{project.summary}</p>
              </td>
              <td className="px-4 py-3 font-mono text-[11px] uppercase tracking-wider text-ink-2">
                {PLATFORM_NAMES[project.platform]}
              </td>
              <td className="px-4 py-3 text-right font-mono text-xs text-ink-2 tabular-nums">
                {formatDownloads(project.downloads)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
