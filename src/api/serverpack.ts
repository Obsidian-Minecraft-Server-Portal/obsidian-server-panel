import { useMutation, useQuery } from "@tanstack/react-query";
import { api } from "./client";
import type { ContentKind, PlatformId } from "./content";
import type { LoaderKind } from "./loaders";

export type DetectedSide = "client" | "server" | "both" | "unsure";

export type LeaningSide = "client" | "server" | "both";

export interface AnalyzedMod {
  index: number;
  filename: string;
  name: string;
  kind: ContentKind;
  side: DetectedSide;
  leaning: LeaningSide | null;
  confidence: number;
  evidence: string[];
  downloadable: boolean;
  remembered: boolean | null;
  has_icon: boolean;
  platform: PlatformId | null;
  project_id: string | null;
  version_id: string | null;
  sha1: string | null;
}

export interface FolderChoice {
  name: string;
  include: boolean;
}

export interface AnalysisResult {
  draft_id: string;
  pack_name: string;
  mc_version: string;
  loader: LoaderKind;
  mods: AnalyzedMod[];
  folders: FolderChoice[];
}

export interface ExportInput {
  name: string;
  accept_eula: boolean;
  mods: { index: number; include: boolean }[];
  folders: { name: string; include: boolean }[];
}

export interface ServerPackReceipt {
  instance_id: number;
  included: number;
  excluded: number;
  warnings: string[];
}

export const draftKey = (draftId: string) => ["serverpack", "draft", draftId] as const;

export function draftIconUrl(draftId: string, index: number): string {
  return `/api/v1/modpacks/drafts/${encodeURIComponent(draftId)}/mods/${index}/icon`;
}

export function useServerPackDraft(draftId: string) {
  return useQuery({
    queryKey: draftKey(draftId),
    queryFn: () => api.get<AnalysisResult>(`/modpacks/drafts/${encodeURIComponent(draftId)}`),
    enabled: draftId !== "",
    staleTime: Infinity,
    retry: false,
  });
}

export function useAnalyzeModpack() {
  return useMutation({
    mutationFn: (input: { platform: PlatformId; projectId: string; versionId: string }) =>
      api.post<{ task_id: number }>("/modpacks/analyze", {
        platform: input.platform,
        project_id: input.projectId,
        version_id: input.versionId,
      }),
  });
}

export function useExportServerPack(draftId: string) {
  return useMutation({
    mutationFn: (input: ExportInput) =>
      api.post<{ task_id: number }>(`/modpacks/drafts/${encodeURIComponent(draftId)}/export`, input),
  });
}
