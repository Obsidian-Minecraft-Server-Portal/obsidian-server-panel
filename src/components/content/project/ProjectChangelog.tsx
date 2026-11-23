import { cn } from "@heroui/react";
import { format } from "date-fns";
import { useState } from "react";
import {
  type PlatformId,
  type ProjectVersion,
  type VersionFilter,
  useProjectChangelog,
  useProjectVersions,
} from "../../../api/content";
import { EmptyState } from "../../ui/EmptyState";

const ALL_VERSIONS: VersionFilter = { loader: null, mcVersion: null };

interface ProjectChangelogProps {
  platform: PlatformId;
  projectId: string;
}

export function ProjectChangelog({ platform, projectId }: ProjectChangelogProps) {
  const versions = useProjectVersions(platform, projectId, ALL_VERSIONS);
  const [picked, setPicked] = useState<string | null>(null);

  if (versions.isPending) return <p className="text-sm text-ink-3">Loading versions…</p>;
  if (versions.isError) return <p className="text-sm text-rose">{versions.error.message}</p>;

  const list = versions.data ?? [];
  const selected = list.find((entry) => entry.id === picked) ?? list[0];
  if (!selected) {
    return (
      <EmptyState icon="lucide:history" title="No versions" description="This project has no published versions yet." />
    );
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
      <ol className="max-h-[70vh] space-y-1.5 overflow-y-auto pr-1">
        {list.map((entry) => (
          <VersionButton
            key={entry.id}
            version={entry}
            active={entry.id === selected.id}
            onPress={() => setPicked(entry.id)}
          />
        ))}
      </ol>
      <Notes platform={platform} projectId={projectId} version={selected} />
    </div>
  );
}

function VersionButton({ version, active, onPress }: { version: ProjectVersion; active: boolean; onPress: () => void }) {
  return (
    <li>
      <button
        type="button"
        aria-current={active}
        onClick={onPress}
        className={cn(
          "w-full rounded-control border px-3 py-2.5 text-left transition-colors",
          active ? "border-accent/50 bg-accent-tint" : "border-line bg-input/60 hover:border-line-strong",
        )}
      >
        <span className="block truncate text-sm font-medium text-ink-0">{version.name || version.version_number}</span>
        <span className="block truncate font-mono text-[11px] text-ink-3">
          {version.channel} · {format(new Date(version.published_at), "d MMM yyyy")}
        </span>
      </button>
    </li>
  );
}

function Notes({ platform, projectId, version }: { platform: PlatformId; projectId: string; version: ProjectVersion }) {
  const changelog = useProjectChangelog(platform, projectId, version.id);
  return (
    <section className="min-w-0 space-y-4">
      <header className="space-y-1">
        <h2 className="text-xl font-semibold text-ink-0">{version.name || version.version_number}</h2>
        <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">
          {version.channel} · {format(new Date(version.published_at), "d MMM yyyy")}
        </p>
      </header>
      {changelog.isPending && <p className="text-sm text-ink-3">Loading changelog…</p>}
      {changelog.isError && <p className="text-sm text-rose">{changelog.error.message}</p>}
      {changelog.data ? (
        <div className="rendered-html" dangerouslySetInnerHTML={{ __html: changelog.data }} />
      ) : (
        changelog.isSuccess && (
          <EmptyState
            icon="lucide:file-x"
            title="No changelog"
            description="This version was published without release notes."
          />
        )
      )}
    </section>
  );
}
