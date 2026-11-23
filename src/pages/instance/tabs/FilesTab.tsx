import { useCallback, useMemo, useState, type ReactNode } from "react";
import { useSearchParams } from "react-router";
import { downloadUrl, useDirectory, type FileEntry } from "../../../api/files";
import { useInstance } from "../../../api/instances";
import { Breadcrumbs } from "../../../components/files/Breadcrumbs";
import { ContextMenu } from "../../../components/ui/ContextMenu";
import { FileDialogs, type FileDialog } from "../../../components/files/FileDialogs";
import { FileDropZone } from "../../../components/files/FileDropZone";
import { FileEditor } from "../../../components/files/FileEditor";
import { FileTable } from "../../../components/files/FileTable";
import { FileToolbar } from "../../../components/files/FileToolbar";
import { SftpCard } from "../../../components/files/SftpCard";
import { UploadPanel } from "../../../components/files/UploadPanel";
import { actionsFor, type FileActionHandlers } from "../../../components/files/fileActions";
import { errorMessage, normalizeDir, parentPath, triggerDownload } from "../../../components/files/fileUtils";
import { useFileOps } from "../../../components/files/useFileOps";
import { useSelection } from "../../../components/files/useSelection";
import { uploadQueue } from "../../../components/files/useUploads";
import { EmptyState } from "../../../components/ui/EmptyState";
import { GhostButton } from "../../../components/ui/GhostButton";
import { useInstanceId } from "../../../components/instance/useInstanceId";

interface ContextTarget {
  x: number;
  y: number;
  entries: FileEntry[];
}

const NO_ENTRIES: FileEntry[] = [];

export function FilesTab() {
  const id = useInstanceId();
  const instance = useInstance(id);
  const [params, setParams] = useSearchParams();
  const dir = normalizeDir(params.get("path") ?? "");
  const listing = useDirectory(id, dir);
  const entries = listing.data ?? NO_ENTRIES;
  const order = useMemo(() => entries.map((entry) => entry.path), [entries]);
  const { selected, pick, focusOnly } = useSelection(order);
  const selectedEntries = useMemo(() => entries.filter((entry) => selected.has(entry.path)), [entries, selected]);
  const ops = useFileOps(id);

  const [editing, setEditing] = useState<string | null>(null);
  const [dialog, setDialog] = useState<FileDialog | null>(null);
  const [target, setTarget] = useState<ContextTarget | null>(null);
  const [error, setError] = useState<string | null>(null);

  const navigate = useCallback(
    (path: string) =>
      setParams((previous) => {
        const next = new URLSearchParams(previous);
        if (path) next.set("path", path);
        else next.delete("path");
        return next;
      }),
    [setParams],
  );
  const closeMenu = useCallback(() => setTarget(null), []);
  const closeDialog = useCallback(() => setDialog(null), []);
  const openFile = useCallback((path: string) => setEditing(path), []);

  const upload = useCallback(
    (files: File[]) => {
      const existing = new Set(entries.filter((entry) => !entry.is_dir).map((entry) => entry.name));
      const clashes = files.filter((file) => existing.has(file.name)).map((file) => file.name);
      const enqueue = () => uploadQueue.enqueue(id, dir, files);
      if (clashes.length > 0) setDialog({ kind: "overwrite", names: clashes, confirm: enqueue });
      else enqueue();
    },
    [entries, id, dir],
  );

  const attempt = useCallback(async (task: () => Promise<unknown>) => {
    setError(null);
    try {
      await task();
    } catch (failure) {
      setError(errorMessage(failure));
    }
  }, []);

  const handlers = useMemo<FileActionHandlers>(
    () => ({
      open: (entry) => (entry.is_dir ? navigate(entry.path) : openFile(entry.path)),
      edit: (entry) => openFile(entry.path),
      download: (entry) => triggerDownload(downloadUrl(id, entry.path)),
      rename: (entry) => setDialog({ kind: "rename", entry }),
      move: (picked) => setDialog({ kind: "move", entries: picked }),
      archive: (picked) => setDialog({ kind: "archive", entries: picked }),
      extract: (entry) => void attempt(() => ops.extract(entry.path)),
      remove: (picked) => setDialog({ kind: "delete", entries: picked }),
    }),
    [id, navigate, openFile, attempt, ops],
  );

  const onContextMenu = useCallback(
    (entry: FileEntry, x: number, y: number) => {
      const inSelection = selected.has(entry.path);
      if (!inSelection) focusOnly(entry.path);
      setTarget({ x, y, entries: inSelection ? selectedEntries : [entry] });
    },
    [selected, focusOnly, selectedEntries],
  );

  const onOpen = useCallback((entry: FileEntry) => handlers.open(entry), [handlers]);
  const toolbarActions = useMemo(() => actionsFor(selectedEntries, handlers), [selectedEntries, handlers]);
  const menuActions = useMemo(() => (target ? actionsFor(target.entries, handlers) : []), [target, handlers]);

  if (editing) {
    return (
      <div className="grid min-h-[30rem] flex-1 items-start gap-5 px-8 pb-8 lg:grid-cols-[18rem_minmax(0,1fr)] lg:grid-rows-1 lg:items-stretch">
        <section className="flex max-h-[50vh] min-h-0 flex-col overflow-hidden rounded-panel border border-line bg-panel lg:max-h-none">
          <div className="shrink-0 border-b border-line p-3">
            <Breadcrumbs path={dir} onNavigate={navigate} />
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto">
            <Listing
              isPending={listing.isPending}
              error={listing.isError ? errorMessage(listing.error) : null}
              isEmpty={listing.isSuccess && entries.length === 0}
              parent={dir ? parentPath(dir) : null}
              onNavigate={navigate}
            >
              <FileTable
                entries={entries}
                selected={new Set([editing])}
                onPick={pick}
                onOpen={onOpen}
                onContextMenu={onContextMenu}
                compact
              />
            </Listing>
          </div>
        </section>
        <div className="flex min-h-0 min-w-0 flex-col">
          <FileEditor key={editing} instanceId={id} path={editing} ops={ops} onClose={() => setEditing(null)} />
        </div>
        {dialog && <FileDialogs dialog={dialog} dir={dir} entries={entries} ops={ops} onOpenFile={openFile} onClose={closeDialog} />}
        {target && <ContextMenu x={target.x} y={target.y} label="File actions" actions={menuActions} onClose={closeMenu} />}
        <UploadPanel />
      </div>
    );
  }

  return (
    <div className="grid items-start gap-5 px-8 pb-8 xl:grid-cols-[1fr_20rem]">
      <FileDropZone label={dir ? `/${dir}` : "the instance root"} onFiles={upload}>
        <section className="overflow-hidden rounded-panel border border-line bg-panel">
          <div className="space-y-3 border-b border-line p-4">
            <FileToolbar
              actions={toolbarActions}
              refreshing={listing.isFetching}
              onNewFolder={() => setDialog({ kind: "newFolder" })}
              onNewFile={() => setDialog({ kind: "newFile" })}
              onRefresh={() => void listing.refetch()}
              onUpload={upload}
            />
            <Breadcrumbs path={dir} onNavigate={navigate} />
          </div>
          {error && <p className="border-b border-line px-4 py-2 text-sm text-rose">{error}</p>}
          <Listing
            isPending={listing.isPending}
            error={listing.isError ? errorMessage(listing.error) : null}
            isEmpty={listing.isSuccess && entries.length === 0}
            parent={dir ? parentPath(dir) : null}
            onNavigate={navigate}
          >
            <FileTable
              entries={entries}
              selected={selected}
              onPick={pick}
              onOpen={onOpen}
              onContextMenu={onContextMenu}
            />
          </Listing>
        </section>
      </FileDropZone>

      <div className="space-y-5">
        <SftpCard instanceId={id} instanceName={instance.data?.name ?? ""} />
      </div>

      {dialog && <FileDialogs dialog={dialog} dir={dir} entries={entries} ops={ops} onOpenFile={openFile} onClose={closeDialog} />}
      {target && <ContextMenu x={target.x} y={target.y} label="File actions" actions={menuActions} onClose={closeMenu} />}
      <UploadPanel />
    </div>
  );
}

interface ListingProps {
  isPending: boolean;
  error: string | null;
  isEmpty: boolean;
  parent: string | null;
  onNavigate: (path: string) => void;
  children: ReactNode;
}

function Listing({ isPending, error, isEmpty, parent, onNavigate, children }: ListingProps) {
  if (isPending) return <p className="p-6 text-sm text-ink-3">Loading files…</p>;
  if (error) {
    return (
      <EmptyState
        icon="lucide:folder-x"
        title="This folder could not be opened"
        description={error}
        action={parent !== null ? <GhostButton onPress={() => onNavigate(parent)}>Go to parent folder</GhostButton> : undefined}
      />
    );
  }
  if (isEmpty) {
    return <EmptyState icon="lucide:folder-open" title="This folder is empty" description="Drop files anywhere here, or use Upload." />;
  }
  return <>{children}</>;
}
