import { fromEvent, type FileWithPath } from "file-selector";

export interface UploadEntry {
  file: File;
  relativePath: string;
}

export async function readUploadEntries(event: Event): Promise<UploadEntry[]> {
  const items = await fromEvent(event);
  return items.flatMap((item) => (item instanceof File ? [toEntry(item)] : []));
}

function toEntry(file: File): UploadEntry {
  const path = (file as FileWithPath).relativePath || file.name;
  return { file, relativePath: path.replace(/^\.?\/+/, "") };
}
