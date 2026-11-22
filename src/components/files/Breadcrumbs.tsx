import { Icon } from "@iconify-icon/react";
import { Fragment } from "react";
import { GhostButton } from "../ui/GhostButton";
import { crumbsFor } from "./fileUtils";

interface BreadcrumbsProps {
  path: string;
  onNavigate: (path: string) => void;
}

export function Breadcrumbs({ path, onNavigate }: BreadcrumbsProps) {
  const crumbs = crumbsFor(path);
  return (
    <nav aria-label="Folder path" className="flex min-w-0 flex-wrap items-center gap-1 font-mono text-xs">
      <GhostButton size="sm" onPress={() => onNavigate("")} aria-label="Instance root">
        <Icon icon="lucide:hard-drive" width={14} height={14} />
        root
      </GhostButton>
      {crumbs.map((crumb) => (
        <Fragment key={crumb.path}>
          <Icon icon="lucide:chevron-right" className="text-ink-4" width={12} height={12} />
          <GhostButton size="sm" onPress={() => onNavigate(crumb.path)}>
            {crumb.label}
          </GhostButton>
        </Fragment>
      ))}
    </nav>
  );
}
