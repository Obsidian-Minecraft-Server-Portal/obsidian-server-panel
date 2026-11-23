import { useState } from "react";
import {
  type ContentKind,
  type Project,
  type ProjectVersion,
  type VersionFilter,
  useProjectVersions,
} from "../../../api/content";
import type { InstanceView } from "../../../api/instances";
import { CheckboxRow } from "../../ui/CheckboxRow";
import { EmptyState } from "../../ui/EmptyState";
import { GhostButton } from "../../ui/GhostButton";
import { SearchSelect } from "../../ui/SearchSelect";
import { SectionTabs, type SectionTab } from "../../ui/SectionTabs";
import type { InstallAction } from "../InstallTargetModal";
import { LOADER_BOUND_KINDS } from "../meta";
import type { ModpackAction } from "../ModpackModal";
import { VersionRow } from "../VersionRow";
import type { QuickAdd } from "./ProjectActions";

type Channel = "all" | "release" | "beta" | "alpha";

const CHANNEL_TABS: SectionTab<Channel>[] = [
  { id: "all", label: "All" },
  { id: "release", label: "Release" },
  { id: "beta", label: "Beta" },
  { id: "alpha", label: "Alpha" },
];

const PAGE_SIZE = 20;

interface ProjectVersionsProps {
  project: Project;
  gameVersions: string[];
  target: InstanceView | null;
  quickAdd: QuickAdd;
  onModpack: (action: ModpackAction) => void;
  onInstall: (action: InstallAction) => void;
}

export function ProjectVersions({ project, gameVersions, target, quickAdd, onModpack, onInstall }: ProjectVersionsProps) {
  const [channel, setChannel] = useState<Channel>("all");
  const [gameVersion, setGameVersion] = useState<string | null>(null);
  const [matchTarget, setMatchTarget] = useState(true);
  const [limit, setLimit] = useState(PAGE_SIZE);
  const isModpack = project.kind === "modpack";
  const loaderBound = LOADER_BOUND_KINDS.has(project.kind);
  const versions = useProjectVersions(
    project.platform,
    project.id,
    versionFilter(project.kind, !isModpack && matchTarget ? target : null, gameVersion),
  );
  const gameVersionOptions = [...gameVersions]
    .sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
    .map((version) => ({ id: version, label: version }));
  const listed = (versions.data ?? []).filter((version) => channel === "all" || version.channel === channel);
  const shown = listed.slice(0, limit);

  const installing = (version: ProjectVersion) => quickAdd.isPending && quickAdd.variables?.versionId === version.id;

  const install = (version: ProjectVersion) => {
    if (isModpack) onModpack({ project, version });
    else if (loaderBound) quickAdd.mutate({ platform: project.platform, projectId: project.id, versionId: version.id });
    else onInstall({ project, version, kind: project.kind });
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end gap-4">
        <SearchSelect
          label="Minecraft version"
          options={gameVersionOptions}
          value={gameVersion}
          onChange={setGameVersion}
          placeholder="Any version"
          isDisabled={gameVersionOptions.length === 0}
          isClearable
          className="w-44"
        />
        <SectionTabs
          label="Release channel"
          tabs={CHANNEL_TABS}
          value={channel}
          onChange={setChannel}
          className="max-w-full overflow-x-auto"
        />
        {!isModpack && (
          <CheckboxRow isSelected={matchTarget} onChange={setMatchTarget} isDisabled={target === null}>
            {target ? `Only versions that match ${target.name}` : "Only versions that match the chosen instance"}
          </CheckboxRow>
        )}
      </div>
      {loaderBound && target === null && (
        <p className="text-sm text-ink-3">Choose an instance above to install this {project.kind}.</p>
      )}
      {versions.isPending && <p className="text-sm text-ink-3">Loading versions…</p>}
      {versions.isError && <p className="text-sm text-rose">{versions.error.message}</p>}
      {versions.isSuccess && listed.length === 0 && (
        <EmptyState icon="lucide:list-x" title="No versions" description="Nothing matches these filters." />
      )}
      {shown.length > 0 && (
        <div className="space-y-2.5">
          {shown.map((version) => (
            <VersionRow
              key={version.id}
              version={version}
              actionLabel={actionLabel(isModpack, installing(version))}
              actionDisabled={(loaderBound && target === null) || installing(version)}
              onAction={() => install(version)}
            />
          ))}
          {listed.length > shown.length && (
            <div className="flex justify-center">
              <GhostButton onPress={() => setLimit((count) => count + PAGE_SIZE)}>Show more</GhostButton>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function versionFilter(kind: ContentKind, target: InstanceView | null, gameVersion: string | null): VersionFilter {
  return {
    loader: target !== null && LOADER_BOUND_KINDS.has(kind) ? target.loader : null,
    mcVersion: gameVersion ?? target?.mc_version ?? null,
  };
}

function actionLabel(isModpack: boolean, installing: boolean): string {
  if (isModpack) return "Create instance";
  return installing ? "Installing…" : "Install";
}
