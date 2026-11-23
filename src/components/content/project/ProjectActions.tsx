import { Icon } from "@iconify-icon/react";
import type { Project, useQuickAdd } from "../../../api/content";
import type { InstanceView } from "../../../api/instances";
import { LOADER_META } from "../../runtime/loaderMeta";
import { OptionSelect } from "../../runtime/OptionSelect";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { LOADER_BOUND_KINDS, PLATFORM_NAMES, httpUrl } from "../meta";

export type QuickAdd = ReturnType<typeof useQuickAdd>;

interface ProjectActionsProps {
  project: Project;
  webUrl: string | null;
  installTargets: InstanceView[];
  isLoading: boolean;
  target: InstanceView | null;
  onTargetChange: (id: number | null) => void;
  quickAdd: QuickAdd;
  onChooseVersion: () => void;
}

export function ProjectActions({
  project,
  webUrl,
  installTargets,
  isLoading,
  target,
  onTargetChange,
  quickAdd,
  onChooseVersion,
}: ProjectActionsProps) {
  const options = installTargets.map((entry) => ({
    id: String(entry.id),
    label: `${entry.name} · ${LOADER_META[entry.loader].label} ${entry.mc_version}`,
  }));
  const external = httpUrl(webUrl);
  const placeholder = isLoading
    ? "Loading instances…"
    : options.length === 0
      ? "No compatible instance"
      : "Choose an instance";

  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-end">
      <OptionSelect
        label="Install to"
        options={options}
        value={target === null ? null : String(target.id)}
        onChange={(value) => onTargetChange(value === null ? null : Number(value))}
        placeholder={placeholder}
        isDisabled={isLoading || options.length === 0}
        className="w-full sm:w-64"
      />
      {LOADER_BOUND_KINDS.has(project.kind) ? (
        <PrimaryButton
          isDisabled={target === null || quickAdd.isPending}
          onPress={() => quickAdd.mutate({ platform: project.platform, projectId: project.id })}
        >
          <Icon icon="lucide:zap" width={14} height={14} />
          {quickAdd.isPending ? "Adding…" : "Quick add"}
        </PrimaryButton>
      ) : (
        <PrimaryButton onPress={onChooseVersion}>
          <Icon icon="lucide:list" width={14} height={14} />
          Choose a version
        </PrimaryButton>
      )}
      {external && (
        <a
          href={external}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex h-10 items-center justify-center gap-2 rounded-control border border-line-strong px-4 text-sm font-medium text-ink-1 transition-colors hover:bg-raised"
        >
          <Icon icon="lucide:external-link" width={14} height={14} />
          Open on {PLATFORM_NAMES[project.platform]}
        </a>
      )}
    </div>
  );
}
