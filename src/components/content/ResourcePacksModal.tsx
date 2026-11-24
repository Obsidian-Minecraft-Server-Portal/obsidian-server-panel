import { Modal, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { DndContext, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent } from "@dnd-kit/core";
import { SortableContext, arrayMove, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import copyToClipboard from "copy-to-clipboard";
import { useRef } from "react";
import { useNavigate } from "react-router";
import {
  type ResourcePack,
  useReorderResourcePacks,
  useRemoveResourcePack,
  useResourcePacks,
  useUploadResourcePack,
} from "../../api/resourcePacks";
import { formatBytes } from "../files/fileUtils";
import { Chip } from "../ui/Chip";
import { EmptyState } from "../ui/EmptyState";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { PLATFORM_NAMES } from "./meta";

interface ResourcePacksModalProps {
  instanceId: number;
  open: boolean;
  canInstall: boolean;
  canRemove: boolean;
  onClose: () => void;
}

export function ResourcePacksModal({ instanceId, open, canInstall, canRemove, onClose }: ResourcePacksModalProps) {
  return (
    <Modal.Backdrop isOpen={open} onOpenChange={(next) => !next && onClose()}>
      <Modal.Container size="lg">
        <Modal.Dialog className="bg-panel text-ink-1">
          {open && <ResourcePacksBody instanceId={instanceId} canInstall={canInstall} canRemove={canRemove} />}
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}

function ResourcePacksBody({ instanceId, canInstall, canRemove }: Omit<ResourcePacksModalProps, "open" | "onClose">) {
  const navigate = useNavigate();
  const packs = useResourcePacks(instanceId);
  const upload = useUploadResourcePack(instanceId);
  const reorder = useReorderResourcePacks(instanceId);
  const remove = useRemoveResourcePack(instanceId);
  const input = useRef<HTMLInputElement>(null);
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );
  const list = packs.data;
  const busy = upload.isPending || reorder.isPending || remove.isPending;
  const error = packs.error ?? upload.error ?? reorder.error ?? remove.error;

  const onDragEnd = (event: DragEndEvent) => {
    if (!list || !event.over || event.active.id === event.over.id) return;
    const ids = list.packs.map((pack) => pack.id);
    const from = ids.indexOf(Number(event.active.id));
    const to = ids.indexOf(Number(event.over.id));
    if (from !== -1 && to !== -1) reorder.mutate(arrayMove(ids, from, to));
  };

  return (
    <>
      <Modal.Header className="flex items-start gap-3">
        <span className="grid size-10 shrink-0 place-items-center rounded-control bg-secondary-tint text-secondary">
          <Icon icon="lucide:palette" width={18} height={18} />
        </span>
        <div className="min-w-0 space-y-1">
          <Modal.Heading className="text-lg font-semibold text-ink-0">Server resource packs</Modal.Heading>
          <p className="text-sm text-ink-3">Packs higher in the list override the ones below. Players get one combined download.</p>
        </div>
      </Modal.Header>
      <Modal.Body className="space-y-4">
        {list && !list.public_url_configured && (
          <p className="flex items-start gap-2 rounded-control border border-line-strong bg-input px-3 py-2 text-sm text-amber">
            <Icon icon="lucide:triangle-alert" width={16} height={16} className="mt-px shrink-0" />
            No public URL is set, so the pack link uses the address this browser opened. Set a public URL in Settings so players can reach it.
          </p>
        )}
        <div className="flex flex-wrap items-center gap-2">
          <PrimaryButton size="sm" isDisabled={!canInstall || busy} onPress={() => input.current?.click()}>
            <Icon icon="lucide:upload" width={15} height={15} />
            Upload .zip
          </PrimaryButton>
          <GhostButton
            size="sm"
            isDisabled={!canInstall}
            onPress={() => navigate(`/discover?${new URLSearchParams({ kind: "resourcepack", instance: String(instanceId) })}`)}
          >
            <Icon icon="lucide:compass" width={15} height={15} />
            Add from Discover
          </GhostButton>
          {list?.url && (
            <GhostButton size="sm" onPress={() => void copyToClipboard(list.url ?? "", { fallbackToPrompt: true })}>
              <Icon icon="lucide:link" width={15} height={15} />
              Copy pack URL
            </GhostButton>
          )}
          <input
            ref={input}
            type="file"
            accept=".zip,application/zip"
            hidden
            onChange={(event) => {
              const file = event.target.files?.[0];
              event.target.value = "";
              if (file) upload.mutate(file);
            }}
          />
        </div>
        {error && <p className="text-sm text-rose">{error.message}</p>}
        {packs.isPending && <p className="text-sm text-ink-3">Loading packs…</p>}
        {list && list.packs.length === 0 && (
          <EmptyState
            icon="lucide:palette"
            title="No resource packs"
            description="Add packs from Discover or upload a .zip file."
          />
        )}
        {list && list.packs.length > 0 && (
          <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
            <SortableContext items={list.packs.map((pack) => pack.id)} strategy={verticalListSortingStrategy}>
              <div className="space-y-2.5">
                {list.packs.map((pack, index) => (
                  <SortablePack
                    key={pack.id}
                    pack={pack}
                    index={index}
                    disabled={!canInstall || busy}
                    canRemove={canRemove && !busy}
                    onRemove={() => remove.mutate(pack.id)}
                  />
                ))}
              </div>
            </SortableContext>
          </DndContext>
        )}
      </Modal.Body>
    </>
  );
}

function SortablePack({
  pack,
  index,
  disabled,
  canRemove,
  onRemove,
}: {
  pack: ResourcePack;
  index: number;
  disabled: boolean;
  canRemove: boolean;
  onRemove: () => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: pack.id, disabled });
  const source = pack.source ? PLATFORM_NAMES[pack.source.platform] : null;

  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={cn(
        "flex items-center gap-3 rounded-control border bg-input/60 p-3",
        isDragging ? "border-accent" : "border-line",
      )}
    >
      <button
        type="button"
        aria-label={`Move ${pack.name}`}
        disabled={disabled}
        className="grid size-8 shrink-0 cursor-grab place-items-center rounded-control text-ink-3 transition-colors hover:bg-raised hover:text-ink-1 disabled:cursor-not-allowed disabled:opacity-40"
        {...attributes}
        {...listeners}
      >
        <Icon icon="lucide:grip-vertical" width={18} height={18} />
      </button>
      <span className="w-5 font-mono text-xs text-ink-3">{index + 1}</span>
      <div className="min-w-0 flex-1 space-y-1.5">
        <p className="truncate font-medium text-ink-0">{pack.name}</p>
        <div className="flex flex-wrap gap-1.5">
          {source ? <Chip tone="secondary">{source}</Chip> : <Chip>Uploaded</Chip>}
          <Chip>{formatBytes(pack.size)}</Chip>
        </div>
      </div>
      {canRemove && (
        <GhostButton size="sm" onPress={onRemove}>
          <Icon icon="lucide:trash-2" width={15} height={15} />
          Remove
        </GhostButton>
      )}
    </div>
  );
}
