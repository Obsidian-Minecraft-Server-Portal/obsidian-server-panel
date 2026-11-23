export function mergeEdits<T extends object>(previous: T, current: T, next: T): T {
  const merged = { ...next };
  for (const key of Object.keys(current) as (keyof T)[]) {
    if (JSON.stringify(current[key]) !== JSON.stringify(previous[key])) merged[key] = current[key];
  }
  return merged;
}
