import type { AnalyzedMod, DetectedSide } from "../../api/serverpack";

export const SIDE_LABELS: Record<DetectedSide, string> = {
  client: "Client",
  server: "Server",
  both: "Both",
  unsure: "Unsure",
};

export function autoInclude(mod: AnalyzedMod): boolean {
  return mod.downloadable && mod.side !== "client";
}

export function effectiveInclude(mod: AnalyzedMod, override: boolean | undefined): boolean {
  return override ?? mod.remembered ?? autoInclude(mod);
}
