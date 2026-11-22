import { useSyncExternalStore } from "react";
import { queryClient } from "../../api/queryClient";
import { filesKey } from "../../api/files";
import { joinPath, errorMessage } from "./fileUtils";
import { uploadFile } from "./uploadFile";

export type UploadState = "queued" | "uploading" | "done" | "failed" | "cancelled";

export interface UploadItem {
  key: string;
  name: string;
  path: string;
  size: number;
  progress: number;
  state: UploadState;
  error?: string;
}

interface Job {
  item: UploadItem;
  instanceId: number;
  file: File;
}

const CONCURRENCY = 2;
const listeners = new Set<() => void>();
const queued: Job[] = [];
const controllers = new Map<string, AbortController>();
let uploads: UploadItem[] = [];
let running = 0;

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot(): UploadItem[] {
  return uploads;
}

function update(key: string, changes: Partial<UploadItem>): void {
  uploads = uploads.map((item) => (item.key === key ? { ...item, ...changes } : item));
  listeners.forEach((listener) => listener());
}

function pump(): void {
  while (running < CONCURRENCY && queued.length > 0) start(queued.shift()!);
}

function start({ item, instanceId, file }: Job): void {
  const controller = new AbortController();
  controllers.set(item.key, controller);
  running += 1;
  update(item.key, { state: "uploading" });

  uploadFile({
    instanceId,
    path: item.path,
    file,
    signal: controller.signal,
    onProgress: (progress) => update(item.key, { progress }),
  })
    .then(
      () => update(item.key, { state: "done", progress: 1 }),
      (error: unknown) =>
        update(
          item.key,
          controller.signal.aborted
            ? { state: "cancelled" }
            : { state: "failed", error: errorMessage(error, "Upload failed") },
        ),
    )
    .finally(() => {
      controllers.delete(item.key);
      running -= 1;
      void queryClient.invalidateQueries({ queryKey: filesKey(instanceId) });
      pump();
    });
}

function enqueue(instanceId: number, dir: string, files: File[]): void {
  const jobs = files.map((file): Job => ({
    instanceId,
    file,
    item: {
      key: crypto.randomUUID(),
      name: file.name,
      path: joinPath(dir, file.name),
      size: file.size,
      progress: 0,
      state: "queued",
    },
  }));
  uploads = [...uploads, ...jobs.map((job) => job.item)];
  queued.push(...jobs);
  listeners.forEach((listener) => listener());
  pump();
}

function cancel(key: string): void {
  const index = queued.findIndex((job) => job.item.key === key);
  if (index !== -1) {
    queued.splice(index, 1);
    update(key, { state: "cancelled" });
  } else {
    controllers.get(key)?.abort();
  }
}

function clearFinished(): void {
  uploads = uploads.filter((item) => item.state === "queued" || item.state === "uploading");
  listeners.forEach((listener) => listener());
}

export const uploadQueue = { enqueue, cancel, clearFinished };

export function useUploadItems(): UploadItem[] {
  return useSyncExternalStore(subscribe, getSnapshot);
}
