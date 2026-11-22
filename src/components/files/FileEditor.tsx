import type { EditorProps } from "@monaco-editor/react";
import { Icon } from "@iconify-icon/react";
import { lazy, Suspense, useCallback, useEffect, useState, type ReactNode } from "react";
import { downloadUrl, useTextFile } from "../../api/files";
import { Chip } from "../ui/Chip";
import { EmptyState } from "../ui/EmptyState";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { ConfirmDialog } from "./ConfirmDialog";
import { baseName, editorLanguage, errorMessage, triggerDownload } from "./fileUtils";
import { defineObsidianTheme, OBSIDIAN_THEME } from "./monacoTheme";
import type { FileOps } from "./useFileOps";

const MonacoEditor = lazy(() => import("./MonacoEditor"));

const EDITOR_OPTIONS: EditorProps["options"] = {
  minimap: { enabled: false },
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: 13,
  scrollBeyondLastLine: false,
  automaticLayout: true,
  wordWrap: "on",
  tabSize: 4,
  smoothScrolling: true,
};

interface FileEditorProps {
  instanceId: number;
  path: string;
  ops: FileOps;
  onClose: () => void;
}

export function FileEditor({ instanceId, path, ops, onClose }: FileEditorProps) {
  const text = useTextFile(instanceId, path);
  const download = () => triggerDownload(downloadUrl(instanceId, path));

  if (text.data) {
    return <EditorBody key={path} path={path} initial={text.data.content} ops={ops} onClose={onClose} onDownload={download} />;
  }

  return (
    <Frame path={path} onClose={onClose} onDownload={download}>
      {text.isError ? (
        <EmptyState
          icon="lucide:file-warning"
          title="This file cannot be edited here"
          description={errorMessage(text.error)}
          action={<GhostButton onPress={download}>Download instead</GhostButton>}
        />
      ) : (
        <p className="p-6 text-sm text-ink-3">Loading file…</p>
      )}
    </Frame>
  );
}

interface EditorBodyProps {
  path: string;
  initial: string;
  ops: FileOps;
  onClose: () => void;
  onDownload: () => void;
}

function EditorBody({ path, initial, ops, onClose, onDownload }: EditorBodyProps) {
  const [saved, setSaved] = useState(initial);
  const [draft, setDraft] = useState(initial);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmDiscard, setConfirmDiscard] = useState(false);
  const dirty = draft !== saved;

  const save = useCallback(async () => {
    if (!dirty || saving) return;
    const content = draft;
    setSaving(true);
    setError(null);
    try {
      await ops.save(path, content);
      setSaved(content);
    } catch (failure) {
      setError(errorMessage(failure));
    } finally {
      setSaving(false);
    }
  }, [dirty, saving, draft, ops, path]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();
        void save();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [save]);

  useEffect(() => {
    if (!dirty) return;
    const onUnload = (event: BeforeUnloadEvent) => event.preventDefault();
    window.addEventListener("beforeunload", onUnload);
    return () => window.removeEventListener("beforeunload", onUnload);
  }, [dirty]);

  const requestClose = () => (dirty ? setConfirmDiscard(true) : onClose());

  return (
    <>
      <Frame
        path={path}
        onClose={requestClose}
        onDownload={onDownload}
        dirty={dirty}
        saving={saving}
        onSave={() => void save()}
        error={error}
      >
        <div className="h-[68vh] min-h-80 lg:h-auto lg:min-h-0 lg:flex-1">
          <Suspense fallback={<p className="p-6 text-sm text-ink-3">Loading editor…</p>}>
            <MonacoEditor
              height="100%"
              language={editorLanguage(baseName(path))}
              theme={OBSIDIAN_THEME}
              value={draft}
              onChange={(value) => setDraft(value ?? "")}
              beforeMount={defineObsidianTheme}
              options={EDITOR_OPTIONS}
            />
          </Suspense>
        </div>
      </Frame>
      {confirmDiscard && (
        <ConfirmDialog
          title="Discard unsaved changes?"
          description={`Edits to ${baseName(path)} since the last save will be lost.`}
          confirmLabel="Discard"
          danger
          onConfirm={async () => onClose()}
          onClose={() => setConfirmDiscard(false)}
        />
      )}
    </>
  );
}

interface FrameProps {
  path: string;
  onClose: () => void;
  onDownload: () => void;
  dirty?: boolean;
  saving?: boolean;
  onSave?: () => void;
  error?: string | null;
  children: ReactNode;
}

function Frame({ path, onClose, onDownload, dirty = false, saving = false, onSave, error, children }: FrameProps) {
  return (
    <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-panel border border-line bg-panel">
      <header className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-line px-5 py-3">
        <div className="flex min-w-0 items-center gap-3">
          <Icon icon="lucide:file-code-2" className="shrink-0 text-secondary" width={18} height={18} />
          <div className="min-w-0">
            <p className="truncate text-sm font-semibold text-ink-0">{baseName(path)}</p>
            <p className="truncate font-mono text-[11px] text-ink-3">{path}</p>
          </div>
          {dirty && <Chip tone="amber" pulse>Unsaved</Chip>}
        </div>
        <div className="flex items-center gap-2">
          {onSave && (
            <PrimaryButton size="sm" isDisabled={!dirty || saving} onPress={onSave}>
              {saving ? "Saving…" : "Save"}
            </PrimaryButton>
          )}
          <GhostButton size="sm" onPress={onDownload}>
            <Icon icon="lucide:download" width={14} height={14} />
            Download
          </GhostButton>
          <GhostButton size="sm" aria-label="Close editor" onPress={onClose}>
            <Icon icon="lucide:x" width={14} height={14} />
          </GhostButton>
        </div>
      </header>
      {error && <p className="border-b border-line px-5 py-2 text-sm text-rose">{error}</p>}
      {children}
    </section>
  );
}
