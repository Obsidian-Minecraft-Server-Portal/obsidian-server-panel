import type { ContentKind, VersionFilter } from "../../api/content";
import type { InstanceView } from "../../api/instances";
import type { SectionTab } from "../ui/SectionTabs";
import { LOADER_BOUND_KINDS } from "./meta";

export type VersionChannel = "all" | "release" | "beta" | "alpha";

export const CHANNEL_TABS: SectionTab<VersionChannel>[] = [
  { id: "all", label: "All" },
  { id: "release", label: "Release" },
  { id: "beta", label: "Beta" },
  { id: "alpha", label: "Alpha" },
];

export function versionFilter(kind: ContentKind, target: InstanceView | null, gameVersion: string | null): VersionFilter {
  return {
    loader: target !== null && LOADER_BOUND_KINDS.has(kind) ? target.loader : null,
    mcVersion: gameVersion ?? target?.mc_version ?? null,
  };
}
