import { useQuery } from "@tanstack/react-query";
import { ApiError, BASE, CSRF_HEADER, api, csrfToken, messageFrom } from "./client";
import { REFRESH_MS } from "./polling";

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  modified: number;
}

export interface TextFile {
  path: string;
  content: string;
  size: number;
}

export interface UploadStatus {
  upload_id: string;
  path: string;
  size: number;
  offset: number;
}

export interface ArchiveStarted {
  task_id: number;
}

export interface ArchiveResult {
  path: string;
  entries: number;
}

export const filesKey = (id: number) => ["instances", id, "files"] as const;
export const directoryKey = (id: number, path: string) => [...filesKey(id), "dir", path] as const;
export const textFileKey = (id: number, path: string) => [...filesKey(id), "text", path] as const;

const filesPath = (id: number) => `/instances/${id}/files`;

function query(params: Record<string, string>): string {
  return new URLSearchParams(params).toString();
}

export const listDirectory = (id: number, path: string) =>
  api.get<FileEntry[]>(`${filesPath(id)}?${query({ path })}`);

export const readText = (id: number, path: string) =>
  api.get<TextFile>(`${filesPath(id)}/content?${query({ path })}`);

export const writeText = (id: number, path: string, content: string) =>
  api.put(`${filesPath(id)}/content`, { path, content });

export const makeDir = (id: number, path: string) => api.post(`${filesPath(id)}/mkdir`, { path });

export const renameEntry = (id: number, from: string, to: string) =>
  api.post(`${filesPath(id)}/rename`, { from, to });

export const deleteEntries = (id: number, paths: string[]) =>
  api.post<{ deleted: number }>(`${filesPath(id)}/delete`, { paths });

export const archiveEntries = (id: number, paths: string[], dest: string) =>
  api.post<ArchiveStarted>(`${filesPath(id)}/archive`, { paths, dest });

export const extractArchive = (id: number, path: string) =>
  api.post<ArchiveResult>(`${filesPath(id)}/extract`, { path });

export const downloadUrl = (id: number, path: string) => `${BASE}${filesPath(id)}/download?${query({ path })}`;

export const selectionUrl = (id: number, dir: string, paths: string[]) =>
  `${BASE}${filesPath(id)}/zip?${query({ dir, paths: JSON.stringify(paths) })}`;

export const createUpload = (id: number, path: string, size: number) =>
  api.post<UploadStatus>(`${filesPath(id)}/uploads`, { path, size });

export const getUpload = (id: number, uploadId: string) =>
  api.get<UploadStatus>(`${filesPath(id)}/uploads/${uploadId}`);

export const finalizeUpload = (id: number, uploadId: string) =>
  api.post<UploadStatus>(`${filesPath(id)}/uploads/${uploadId}/finalize`);

export const cancelUpload = (id: number, uploadId: string) => api.delete(`${filesPath(id)}/uploads/${uploadId}`);

export function sendChunk(
  id: number,
  uploadId: string,
  offset: number,
  chunk: Blob,
  onProgress: (loaded: number) => void,
  signal: AbortSignal,
): Promise<UploadStatus> {
  return new Promise((resolve, reject) => {
    if (signal.aborted) {
      reject(new ApiError(0, "Upload cancelled"));
      return;
    }
    const xhr = new XMLHttpRequest();
    const abort = () => xhr.abort();
    const settle = () => signal.removeEventListener("abort", abort);
    xhr.open("PUT", `${BASE}${filesPath(id)}/uploads/${uploadId}?${query({ offset: String(offset) })}`);
    xhr.withCredentials = true;
    const token = csrfToken();
    if (token) xhr.setRequestHeader(CSRF_HEADER, token);
    xhr.upload.onprogress = (event) => onProgress(event.loaded);
    xhr.onload = () => {
      settle();
      if (xhr.status >= 200 && xhr.status < 300) {
        resolve(JSON.parse(xhr.responseText) as UploadStatus);
      } else {
        reject(new ApiError(xhr.status, messageFrom(xhr.responseText, xhr.statusText)));
      }
    };
    xhr.onerror = () => {
      settle();
      reject(new ApiError(0, "Network error"));
    };
    xhr.onabort = () => {
      settle();
      reject(new ApiError(0, "Upload cancelled"));
    };
    signal.addEventListener("abort", abort, { once: true });
    xhr.send(chunk);
  });
}

export function useDirectory(id: number, path: string) {
  return useQuery({
    queryKey: directoryKey(id, path),
    queryFn: () => listDirectory(id, path),
    refetchInterval: REFRESH_MS.list,
  });
}

export function useTextFile(id: number, path: string) {
  return useQuery({
    queryKey: textFileKey(id, path),
    queryFn: () => readText(id, path),
    staleTime: 0,
    retry: false,
  });
}
