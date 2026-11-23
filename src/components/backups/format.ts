import type { Tone } from "../ui/tone";

const UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), UNITS.length - 1);
  const value = bytes / 1024 ** exponent;
  return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${UNITS[exponent]}`;
}

export function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

export function formatRatio(ratio: number): string {
  return `${ratio.toFixed(1)}×`;
}

export function labelTone(label: string): Tone {
  if (label === "manual") return "secondary";
  if (label === "scheduled") return "secondary";
  return "neutral";
}
