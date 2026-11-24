import { Icon } from "@iconify-icon/react";
import { useRef, useState, type ChangeEvent } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router";
import {
  useApplyContentUpdate,
  useContentUpdates,
  useFindContentMatches,
  useInstalledContent,
  useRemoveContent,
  useToggleContent,
  useUpdateAllContent,
  useUploadContent,
} from "../../../api/content";
import type { ContentKind, InstalledContent } from "../../../api/content";
import { hasPermission, useInstance } from "../../../api/instances";
import { InstalledRow } from "../../../components/content/InstalledRow";
import { RelationsModal, type RelationsTarget } from "../../../components/content/RelationsModal";
import { SwitchVersionModal } from "../../../components/content/SwitchVersionModal";
import { ServerWorldPanel } from "../../../components/content/ServerWorldPanel";
import { kindForLoader } from "../../../components/content/meta";
import { EmptyState } from "../../../components/ui/EmptyState";
import { GhostButton } from "../../../components/ui/GhostButton";
import { PrimaryButton } from "../../../components/ui/PrimaryButton";
import { ProgressBar } from "../../../components/ui/ProgressBar";

const NON_CONTENT_LOADERS = new Set(["vanilla", "custom"]);

const GROUPS: { label: string; kinds: ContentKind[] }[] = [
  { label: "Mods & plugins", kinds: ["mod", "plugin"] },
  { label: "Data packs", kinds: ["datapack"] },
];

function groupItems(items: InstalledContent[]) {
  return GROUPS.map((group) => ({ ...group, items: items.filter((item) => group.kinds.includes(item.kind)) })).filter(
    (group) => group.items.length > 0,
  );
}

function filesLabel(count: number) {
  return `${count} unlinked ${count === 1 ? "file" : "files"}`;
}

export function InstalledContentPanel() {
  const { id = "" } = useParams();
  const instanceId = Number(id);
  const navigate = useNavigate();
  const instance = useInstance(instanceId);
  const installed = useInstalledContent(instanceId);
  const updates = useContentUpdates(instanceId);
  const toggle = useToggleContent(instanceId);
  const applyUpdate = useApplyContentUpdate(instanceId);
  const updateAll = useUpdateAllContent(instanceId);
  const upload = useUploadContent(instanceId);
  const remove = useRemoveContent(instanceId);
  const matches = useFindContentMatches(instanceId);
  const [pendingRemove, setPendingRemove] = useState<number | null>(null);
  const [relations, setRelations] = useState<RelationsTarget | null>(null);
  const [switching, setSwitching] = useState<InstalledContent | null>(null);
  const [searchParams] = useSearchParams();
  const associated = searchParams.get("associated");
  const [uploadProgress, setUploadProgress] = useState<number | null>(null);
  const fileInput = useRef<HTMLInputElement>(null);

  const items = installed.data ?? [];
  const updateList = updates.data ?? [];
  const updateByMod = new Map(updateList.map((entry) => [entry.mod_id, entry]));
  const enabledCount = items.filter((item) => item.enabled).length;
  const unlinkedCount = items.filter((item) => item.platform === null).length;
  const canInstall = instance.data ? hasPermission(instance.data, "content.install") : false;
  const busy = toggle.isPending || applyUpdate.isPending || updateAll.isPending || remove.isPending || matches.isPending;
  const error = matches.error ?? toggle.error ?? applyUpdate.error ?? updateAll.error ?? remove.error ?? upload.error ?? installed.error;

  const browse = () => {
    const params = new URLSearchParams({ kind: kindForLoader(instance.data?.loader ?? "vanilla"), instance: id });
    const loader = instance.data?.loader;
    if (loader && !NON_CONTENT_LOADERS.has(loader)) params.set("loader", loader);
    if (instance.data) params.set("mc_version", instance.data.mc_version);
    navigate(`/discover?${params}`);
  };

  const associate = (item: InstalledContent) =>
    navigate(`/discover?${new URLSearchParams({ kind: item.kind, instance: id, associate: String(item.id) })}`);

  const onFile = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    setUploadProgress(0);
    upload.mutate({ file, onProgress: setUploadProgress }, { onSettled: () => setUploadProgress(null) });
  };

  const removeItem = (modId: number) => {
    if (pendingRemove !== modId) {
      setPendingRemove(modId);
      return;
    }
    remove.mutate(modId, { onSettled: () => setPendingRemove(null) });
  };

  return (
    <div className="space-y-6 px-4 sm:px-8 pb-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">
          {items.length} installed · {enabledCount} enabled
          {updateList.length > 0 && ` · ${updateList.length} updates`}
        </p>
        <div className="flex flex-wrap gap-2">
          <GhostButton onPress={() => void updates.refetch()} isDisabled={updates.isFetching}>
            <Icon icon="lucide:refresh-cw" width={15} height={15} />
            Check updates
          </GhostButton>
          {canInstall && unlinkedCount > 0 && (
            <GhostButton onPress={() => matches.mutate()} isDisabled={busy}>
              <Icon icon="lucide:link" width={15} height={15} />
              {matches.isPending ? "Matching…" : "Find matches"}
            </GhostButton>
          )}
          <GhostButton onPress={browse}>
            <Icon icon="lucide:search" width={15} height={15} />
            Browse
          </GhostButton>
          <GhostButton isDisabled={upload.isPending} onPress={() => fileInput.current?.click()}>
            <Icon icon="lucide:upload" width={15} height={15} />
            Upload jar
          </GhostButton>
          {updateList.length > 0 && (
            <PrimaryButton isDisabled={busy} onPress={() => updateAll.mutate()}>
              <Icon icon="lucide:arrow-up-circle" width={15} height={15} />
              Update all
            </PrimaryButton>
          )}
          <input ref={fileInput} type="file" accept=".jar,application/java-archive" className="hidden" onChange={onFile} />
        </div>
      </div>

      <ServerWorldPanel
        instanceId={instanceId}
        canRemove={instance.data ? hasPermission(instance.data, "content.remove") : false}
      />

      {uploadProgress !== null && (
        <div className="space-y-2">
          <p className="font-mono text-xs text-ink-3">Uploading…</p>
          <ProgressBar value={uploadProgress * 100} />
        </div>
      )}
      {error && <p className="text-sm text-rose">{error.message}</p>}
      {matches.data && (
        <p className={`flex items-center gap-2 text-sm ${matches.data.linked > 0 ? "text-green" : "text-ink-3"}`}>
          <Icon icon={matches.data.linked > 0 ? "lucide:circle-check" : "lucide:search-x"} width={16} height={16} />
          {matches.data.linked > 0
            ? `Linked ${matches.data.linked} of ${filesLabel(matches.data.checked)} to Modrinth or CurseForge.`
            : `No Modrinth or CurseForge match for ${filesLabel(matches.data.checked)}.`}
        </p>
      )}
      {associated && (
        <p className="flex items-center gap-2 text-sm text-green">
          <Icon icon="lucide:circle-check" width={16} height={16} />
          Linked {associated} to its project. Updates now check that project.
        </p>
      )}

      {installed.isPending ? (
        <p className="font-mono text-xs text-ink-3">Loading content…</p>
      ) : items.length === 0 ? (
        <EmptyState
          icon="lucide:puzzle"
          title="No content installed"
          description="Mods, plugins and data packs installed on this instance appear here."
          action={
            <PrimaryButton onPress={browse}>
              <Icon icon="lucide:search" width={15} height={15} />
              Browse content
            </PrimaryButton>
          }
        />
      ) : (
        <div className="space-y-6">
          {groupItems(items).map((group) => (
            <section key={group.label} className="space-y-2.5">
              <h3 className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">
                {group.label} · {group.items.length}
              </h3>
              {group.items.map((item) => (
                <InstalledRow
                  key={item.id}
                  item={item}
                  update={updateByMod.get(item.id)}
                  busy={busy}
                  confirmingRemove={pendingRemove === item.id}
                  onToggle={(enabled) => toggle.mutate({ modId: item.id, enabled })}
                  onApplyUpdate={(entry) => applyUpdate.mutate({ modId: entry.mod_id, versionId: entry.latest.id })}
                  onRemove={() => removeItem(item.id)}
                  onRelations={(tab) => setRelations({ item, tab })}
                  onAssociate={() => associate(item)}
                  onSwitchVersion={() => setSwitching(item)}
                />
              ))}
            </section>
          ))}
        </div>
      )}
      <RelationsModal instanceId={instanceId} target={relations} onClose={() => setRelations(null)} />
      <SwitchVersionModal instance={instance.data ?? null} item={switching} onClose={() => setSwitching(null)} />
    </div>
  );
}
