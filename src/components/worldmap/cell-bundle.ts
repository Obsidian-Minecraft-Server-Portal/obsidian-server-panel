// Decodes the cell bundles the server packs (crates/obsidian-worldmap-service/src/bundle.rs): a count, then for each
// section its chunk x, section y and chunk z, its byte length and its OBSM mesh, all little-endian.

export interface BundleEntry {
  x: number;
  y: number;
  z: number;
  mesh: ArrayBuffer;
}

const ENTRY_HEADER = 16;

export function decodeCellBundle(buffer: ArrayBuffer): BundleEntry[] {
  const view = new DataView(buffer);
  const count = view.getUint32(0, true);
  const entries: BundleEntry[] = [];
  let offset = 4;
  for (let index = 0; index < count; index++) {
    if (offset + ENTRY_HEADER > buffer.byteLength) {
      throw new Error("cell bundle is truncated");
    }
    const start = offset + ENTRY_HEADER;
    const end = start + view.getUint32(offset + 12, true);
    if (end > buffer.byteLength) {
      throw new Error("cell bundle is truncated");
    }
    entries.push({
      x: view.getInt32(offset, true),
      y: view.getInt32(offset + 4, true),
      z: view.getInt32(offset + 8, true),
      mesh: buffer.slice(start, end),
    });
    offset = end;
  }
  return entries;
}
