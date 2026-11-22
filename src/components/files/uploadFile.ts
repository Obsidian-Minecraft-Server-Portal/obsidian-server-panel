import { cancelUpload, createUpload, finalizeUpload, getUpload, sendChunk } from "../../api/files";

const CHUNK_SIZE = 4 * 1024 * 1024;
const MAX_ATTEMPTS = 4;

interface UploadOptions {
  instanceId: number;
  path: string;
  file: File;
  onProgress: (fraction: number) => void;
  signal: AbortSignal;
}

export async function uploadFile({ instanceId, path, file, onProgress, signal }: UploadOptions): Promise<void> {
  const session = await createUpload(instanceId, path, file.size);
  const uploadId = session.upload_id;
  try {
    let offset = session.offset;
    let attempts = 0;
    while (offset < file.size) {
      const start = offset;
      const end = Math.min(start + CHUNK_SIZE, file.size);
      try {
        const status = await sendChunk(
          instanceId,
          uploadId,
          start,
          file.slice(start, end),
          (loaded) => onProgress((start + loaded) / file.size),
          signal,
        );
        offset = status.offset;
        attempts = 0;
      } catch (error) {
        if (signal.aborted || ++attempts >= MAX_ATTEMPTS) throw error;
        offset = (await getUpload(instanceId, uploadId)).offset;
      }
    }
    await finalizeUpload(instanceId, uploadId);
  } catch (error) {
    await cancelUpload(instanceId, uploadId).catch(() => undefined);
    throw error;
  }
}
