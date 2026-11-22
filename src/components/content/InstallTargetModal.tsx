import { Modal } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import type { Project, ProjectVersion } from "../../api/content";
import { useInstallContent } from "../../api/content";
import { useInstances } from "../../api/instances";
import { LOADER_META } from "../runtime/loaderMeta";
import { OptionSelect } from "../runtime/OptionSelect";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";

export interface InstallAction {
  project: Project;
  version: ProjectVersion;
}

interface InstallTargetModalProps {
  action: InstallAction | null;
  defaultInstanceId: number | null;
  onClose: () => void;
}

export function InstallTargetModal({ action, defaultInstanceId, onClose }: InstallTargetModalProps) {
  return (
    <Modal.Backdrop isOpen={action !== null} onOpenChange={(open) => !open && onClose()}>
      <Modal.Container size="md">
        <Modal.Dialog className="bg-panel text-ink-1">
          {action && <InstallForm action={action} defaultInstanceId={defaultInstanceId} onClose={onClose} />}
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

interface InstallFormProps {
  action: InstallAction;
  defaultInstanceId: number | null;
  onClose: () => void;
}

function InstallForm({ action, defaultInstanceId, onClose }: InstallFormProps) {
  const { data: instances, isLoading, isError } = useInstances();
  const [selected, setSelected] = useState<string | null>(defaultInstanceId === null ? null : String(defaultInstanceId));
  const instanceId = selected === null ? 0 : Number(selected);
  const install = useInstallContent(instanceId);
  const installedTo = instances?.find((entry) => entry.id === instanceId);

  const options = (instances ?? []).map((entry) => ({
    id: String(entry.id),
    label: `${entry.name} · ${LOADER_META[entry.loader].label} ${entry.mc_version}`,
  }));

  const submit = () =>
    install.mutate({
      platform: action.project.platform,
      projectId: action.project.id,
      versionId: action.version.id,
    });

  return (
    <>
      <Modal.Header className="flex items-start gap-3">
        <span className="grid size-10 place-items-center rounded-control bg-secondary-tint text-secondary">
          <Icon icon="lucide:download" width={18} height={18} />
        </span>
        <div className="min-w-0 space-y-1">
          <Modal.Heading className="text-lg font-semibold text-ink-0">Install {action.project.name}</Modal.Heading>
          <p className="truncate font-mono text-[11px] text-ink-3">{action.version.file_name}</p>
        </div>
      </Modal.Header>
      <Modal.Body className="space-y-4">
        {install.isSuccess ? (
          <p className="flex items-center gap-2 text-sm text-cyan">
            <Icon icon="lucide:circle-check" width={16} height={16} />
            Installed to {installedTo?.name ?? "the instance"}.
          </p>
        ) : (
          <OptionSelect
            label="Install to instance"
            options={options}
            value={selected}
            onChange={setSelected}
            placeholder={isLoading ? "Loading instances…" : isError ? "Instances could not be loaded" : "Choose an instance"}
            isDisabled={isLoading || isError || options.length === 0 || install.isPending}
          />
        )}
        {install.error && <p className="text-sm text-rose">{install.error.message}</p>}
      </Modal.Body>
      <Modal.Footer className="flex justify-end gap-2">
        <GhostButton onPress={onClose}>
          {install.isSuccess ? "Done" : "Cancel"}
        </GhostButton>
        {!install.isSuccess && (
          <PrimaryButton isDisabled={selected === null || install.isPending} onPress={submit}>
            {install.isPending ? "Installing…" : "Install"}
          </PrimaryButton>
        )}
      </Modal.Footer>
    </>
  );
}
