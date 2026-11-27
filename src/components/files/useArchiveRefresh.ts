import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { filesKey } from "../../api/files";
import { useTasks } from "../../api/tasks";

const ARCHIVE_TASK_KIND = "file_archive";

/** Refreshes the listing when a zip task ends, even after its dialog has been closed. */
export function useArchiveRefresh(instanceId: number): void {
  const queryClient = useQueryClient();
  const { data } = useTasks();
  const settled = useRef<Set<number> | null>(null);

  useEffect(() => {
    if (!data) return;
    const done = new Set(
      data.filter((task) => task.kind === ARCHIVE_TASK_KIND && task.status !== "running").map((task) => task.id),
    );
    const previous = settled.current;
    settled.current = done;
    if (previous && [...done].some((id) => !previous.has(id))) {
      void queryClient.invalidateQueries({ queryKey: filesKey(instanceId) });
    }
  }, [data, instanceId, queryClient]);
}
