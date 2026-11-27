import { useMemo } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  archiveEntries,
  deleteEntries,
  extractArchive,
  filesKey,
  makeDir,
  renameEntry,
  writeText,
} from "../../api/files";
import { baseName, joinPath } from "./fileUtils";

export interface FileOps {
  createFolder: (path: string) => Promise<void>;
  createFile: (path: string) => Promise<void>;
  rename: (from: string, to: string) => Promise<void>;
  move: (paths: string[], destDir: string) => Promise<void>;
  remove: (paths: string[]) => Promise<void>;
  /** Starts a background zip and resolves with its task id. */
  archive: (paths: string[], dest: string) => Promise<number>;
  extract: (path: string) => Promise<void>;
  save: (path: string, content: string) => Promise<void>;
}

export function useFileOps(instanceId: number): FileOps {
  const queryClient = useQueryClient();

  return useMemo(() => {
    const mutate = async (task: () => Promise<unknown>) => {
      try {
        await task();
      } finally {
        void queryClient.invalidateQueries({ queryKey: filesKey(instanceId) });
      }
    };

    return {
      createFolder: (path) => mutate(() => makeDir(instanceId, path)),
      createFile: (path) => mutate(() => writeText(instanceId, path, "")),
      rename: (from, to) => mutate(() => renameEntry(instanceId, from, to)),
      move: (paths, destDir) =>
        mutate(async () => {
          for (const from of paths) {
            await renameEntry(instanceId, from, joinPath(destDir, baseName(from)));
          }
        }),
      remove: (paths) => mutate(() => deleteEntries(instanceId, paths)),
      archive: (paths, dest) => archiveEntries(instanceId, paths, dest).then((started) => started.task_id),
      extract: (path) => mutate(() => extractArchive(instanceId, path)),
      save: (path, content) => mutate(() => writeText(instanceId, path, content)),
    };
  }, [instanceId, queryClient]);
}
