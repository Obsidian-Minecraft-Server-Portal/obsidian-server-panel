import { useState } from "react";
import { useDetectJava, useJavaRuntimes, useRemoveJava } from "../../api/java";
import { EmptyState } from "../ui/EmptyState";
import { GhostButton } from "../ui/GhostButton";
import { Panel } from "../ui/Panel";
import { RuntimeRow } from "./RuntimeRow";

export function RuntimeTable({ canManage }: { canManage: boolean }) {
  const { data: runtimes, isError, error } = useJavaRuntimes();
  const detect = useDetectJava();
  const remove = useRemoveJava();
  const [confirmId, setConfirmId] = useState<number | null>(null);

  const onRemove = (id: number) => {
    if (confirmId !== id) return setConfirmId(id);
    remove.mutate(id, { onSettled: () => setConfirmId(null) });
  };

  const actions = canManage && (
    <GhostButton size="sm" isPending={detect.isPending} onPress={() => detect.mutate()}>
      Detect system Java
    </GhostButton>
  );

  return (
    <Panel title="Installed runtimes" actions={actions}>
      {isError && <p className="mb-4 text-sm text-rose">{error.message}</p>}
      {(detect.error || remove.error) && (
        <p className="mb-4 text-sm text-rose">{(detect.error ?? remove.error)?.message}</p>
      )}
      {runtimes?.length === 0 && (
        <EmptyState
          icon="lucide:coffee"
          title="No Java runtimes yet"
          description={
            canManage
              ? "Install a managed runtime below, or detect one already on this machine."
              : "An administrator can install or detect Java runtimes."
          }
        />
      )}
      {runtimes?.map((runtime) => (
        <RuntimeRow
          key={runtime.id}
          runtime={runtime}
          canManage={canManage}
          confirming={confirmId === runtime.id}
          removing={remove.isPending && remove.variables === runtime.id}
          onRemove={() => onRemove(runtime.id)}
          onCancel={() => setConfirmId(null)}
        />
      ))}
    </Panel>
  );
}
