import { format } from "date-fns";
import type { ContentKind, ProjectVersion } from "../../api/content";
import { Chip } from "../ui/Chip";
import { PrimaryButton } from "../ui/PrimaryButton";

interface VersionRowProps {
  version: ProjectVersion;
  kind: ContentKind;
  onAction: (version: ProjectVersion) => void;
}

const VISIBLE_MC_VERSIONS = 3;

export function VersionRow({ version, kind, onAction }: VersionRowProps) {
  const mcVersions = version.mc_versions.slice(0, VISIBLE_MC_VERSIONS);
  const hiddenMc = version.mc_versions.length - mcVersions.length;
  const published = format(new Date(version.published_at), "d MMM yyyy");

  return (
    <div className="flex flex-wrap items-center gap-3 rounded-control border border-line bg-input/60 p-3">
      <div className="min-w-0 flex-1 space-y-1.5">
        <p className="truncate font-medium text-ink-0">{version.name || version.version_number}</p>
        <p className="truncate font-mono text-[11px] text-ink-3">
          {version.file_name} · {published}
        </p>
        <div className="flex flex-wrap gap-1.5">
          {mcVersions.map((mc) => (
            <Chip key={mc} tone="secondary">
              {mc}
            </Chip>
          ))}
          {hiddenMc > 0 && <Chip>+{hiddenMc}</Chip>}
          {version.loaders.slice(0, 2).map((loader) => (
            <Chip key={loader} tone="cyan">
              {loader}
            </Chip>
          ))}
        </div>
      </div>
      {version.download_url === null ? (
        <Chip tone="amber">Manual download only</Chip>
      ) : (
        <PrimaryButton size="sm" onPress={() => onAction(version)}>
          {kind === "modpack" ? "Create instance" : "Install"}
        </PrimaryButton>
      )}
    </div>
  );
}
