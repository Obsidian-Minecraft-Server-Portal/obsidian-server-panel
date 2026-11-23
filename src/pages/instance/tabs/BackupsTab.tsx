import { useState } from "react";
import { useParams } from "react-router";
import {
  useBackupEvents,
  useBackupPolicy,
  useBackupStats,
  useCreateBackup,
  useDeleteBackup,
  useRestoreBackup,
  useSnapshots,
  type BackupStarted,
} from "../../../api/backups";
import { useTasks, type TaskView } from "../../../api/tasks";
import { BackupPolicyEditor } from "../../../components/backups/BackupPolicyEditor";
import { SnapshotBrowser } from "../../../components/backups/SnapshotBrowser";
import { SnapshotList } from "../../../components/backups/SnapshotList";
import { StoreStats } from "../../../components/backups/StoreStats";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Panel } from "../../../components/ui/Panel";
import { PrimaryButton } from "../../../components/ui/PrimaryButton";
import { ProgressBar } from "../../../components/ui/ProgressBar";

export function BackupsTab() {
  const { id = "" } = useParams();
  const instanceId = Number(id);
  useBackupEvents(instanceId);

  const snapshots = useSnapshots(instanceId);
  const stats = useBackupStats(instanceId);
  const policy = useBackupPolicy(instanceId);
  const create = useCreateBackup(instanceId);
  const restore = useRestoreBackup(instanceId);
  const remove = useDeleteBackup(instanceId);
  const tasks = useTasks();

  const [browsing, setBrowsing] = useState<string | null>(null);
  const [taskId, setTaskId] = useState<number | null>(null);

  const task = tasks.data?.find((item) => item.id === taskId);
  const busy = create.isPending || restore.isPending || remove.isPending;
  const browsed = snapshots.data?.find((snapshot) => snapshot.id === browsing);
  const failure = create.error ?? restore.error ?? remove.error;

  const trackTask = (started: BackupStarted) => setTaskId(started.task_id);
  const restoreSnapshot = (snapshot: string, path?: string) => restore.mutate({ snapshot, path }, { onSuccess: trackTask });
  const deleteSnapshot = (snapshot: string) =>
    remove.mutate(snapshot, { onSuccess: () => setBrowsing((current) => (current === snapshot ? null : current)) });

  const loadFailure = snapshots.error ?? stats.error ?? policy.error;
  if (loadFailure) {
    return (
      <div className="px-4 sm:px-8 pb-8">
        <EmptyState icon="lucide:archive-x" title="Backups unavailable" description={loadFailure.message} />
      </div>
    );
  }

  return (
    <div className="space-y-6 px-4 sm:px-8 pb-8">
      <Panel
        title="Snapshots"
        actions={
          <PrimaryButton isDisabled={busy} onPress={() => create.mutate(undefined, { onSuccess: trackTask })}>
            Back up now
          </PrimaryButton>
        }
      >
        {stats.data && (
          <div className="mb-4">
            <StoreStats stats={stats.data} />
          </div>
        )}
        {task && <TaskLine task={task} />}
        {failure && <p className="mb-3 text-sm text-rose">{failure.message}</p>}
        {!snapshots.data ? (
          <p className="font-mono text-xs text-ink-3">LOADING SNAPSHOTS…</p>
        ) : snapshots.data.length === 0 ? (
          <EmptyState
            icon="lucide:archive"
            title="No backups yet"
            description="Back up now to take the first snapshot, or add a schedule to back up on a timer."
          />
        ) : (
          <SnapshotList
            instanceId={instanceId}
            snapshots={snapshots.data}
            browsing={browsing}
            disabled={busy}
            onBrowse={setBrowsing}
            onRestore={(snapshot) => restoreSnapshot(snapshot)}
            onDelete={deleteSnapshot}
          />
        )}
      </Panel>

      {browsed && (
        <SnapshotBrowser
          key={browsed.id}
          instanceId={instanceId}
          snapshot={browsed}
          disabled={busy}
          onClose={() => setBrowsing(null)}
          onRestore={(path) => restoreSnapshot(browsed.id, path)}
        />
      )}

      <Panel title="Retention and excludes">
        {policy.data && (
          <BackupPolicyEditor instanceId={instanceId} policy={policy.data} />
        )}
      </Panel>
    </div>
  );
}

function TaskLine({ task }: { task: TaskView }) {
  const running = task.status === "running";
  const text = task.status === "failed" ? (task.error ?? `${task.title} failed`) : task.message || task.title;
  return (
    <div className="mb-4 space-y-2">
      <p className={task.status === "failed" ? "font-mono text-xs text-rose" : "font-mono text-xs text-ink-3"}>
        {task.status === "finished" ? "DONE" : text.toUpperCase()}
      </p>
      {running && <ProgressBar value={task.progress * 100} />}
    </div>
  );
}
