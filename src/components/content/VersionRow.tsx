import { format } from "date-fns";
import type { ProjectVersion } from "../../api/content";
import { Chip } from "../ui/Chip";
import { PrimaryButton } from "../ui/PrimaryButton";
import type { Tone } from "../ui/tone";
import { formatDownloads } from "./meta";

interface VersionRowProps {
  version: ProjectVersion;
  actionLabel: string;
  actionDisabled: boolean;
  onAction: () => void;
}

const VISIBLE_MC_VERSIONS = 3;
const CHANNEL_TONE: Record<string, Tone> = { beta: "amber", alpha: "rose" };

export function VersionRow({ version, actionLabel, actionDisabled, onAction }: VersionRowProps) {
  const mcVersions = version.mc_versions.slice(0, VISIBLE_MC_VERSIONS);
  const hiddenMc = version.mc_versions.length - mcVersions.length;
  const published = format(new Date(version.published_at), "d MMM yyyy");
  const required = version.dependencies.filter((dependency) => dependency.kind === "required").length;

  return (
    <div className="flex flex-wrap items-center gap-3 rounded-control border border-line bg-input/60 p-3">
      <div className="min-w-[14rem] flex-1 space-y-1.5">
        <div className="flex min-w-0 items-center gap-2">
          <p className="truncate font-medium text-ink-0">{version.name || version.version_number}</p>
          {version.channel !== "release" && <Chip tone={CHANNEL_TONE[version.channel] ?? "neutral"}>{version.channel}</Chip>}
        </div>
        <p className="truncate font-mono text-[11px] text-ink-3">
          {version.file_name} · {published}
          {version.downloads > 0 && ` · ${formatDownloads(version.downloads)} downloads`}
        </p>
        <div className="flex flex-wrap gap-1.5">
          {mcVersions.map((mc) => (
            <Chip key={mc} tone="secondary">
              {mc}
            </Chip>
          ))}
          {hiddenMc > 0 && <Chip>+{hiddenMc}</Chip>}
          {version.loaders.slice(0, 2).map((loader) => (
            <Chip key={loader} tone="secondary">
              {loader}
            </Chip>
          ))}
          {version.has_server_pack && <Chip tone="secondary">Server pack</Chip>}
          {required > 0 && (
            <Chip>
              {required} required {required === 1 ? "dependency" : "dependencies"}
            </Chip>
          )}
        </div>
      </div>
      {version.download_url === null ? (
        <Chip tone="amber">Manual download only</Chip>
      ) : (
        <PrimaryButton size="sm" className="w-full sm:w-auto" isDisabled={actionDisabled} onPress={onAction}>
          {actionLabel}
        </PrimaryButton>
      )}
    </div>
  );
}
