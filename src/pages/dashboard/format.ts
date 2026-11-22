const MIB = 1024 * 1024;
const GIB = 1024 * MIB;

export function formatMemory(bytes: number): string {
  return bytes >= GIB ? `${(bytes / GIB).toFixed(1)} GB` : `${Math.round(bytes / MIB)} MB`;
}
