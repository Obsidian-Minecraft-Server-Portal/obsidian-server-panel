import { useMemo } from "react";
import { type PlatformId, useCurrentWorld, useInstalledContent, useResourcePack } from "../../api/content";

export interface InstalledProjects {
  has: (platform: PlatformId, projectId: string) => boolean;
  hasVersion: (platform: PlatformId, projectId: string, versionId: string) => boolean;
}

const NOTHING_INSTALLED: InstalledProjects = { has: () => false, hasVersion: () => false };

export function useInstalledProjects(instanceId: number | null): InstalledProjects {
  const enabled = instanceId !== null;
  const id = instanceId ?? 0;
  const installed = useInstalledContent(id, enabled, false);
  const pack = useResourcePack(id, enabled);
  const world = useCurrentWorld(id, enabled);

  return useMemo(() => {
    if (!enabled) return NOTHING_INSTALLED;
    const projects = new Set<string>();
    const versions = new Set<string>();
    const record = (platform: PlatformId, projectId: string, versionId: string | null) => {
      projects.add(`${platform}:${projectId}`);
      if (versionId) versions.add(`${platform}:${projectId}:${versionId}`);
    };
    for (const item of installed.data ?? []) {
      if (item.platform !== null && item.project_id) record(item.platform, item.project_id, item.version_id);
    }
    for (const source of [pack.data?.source, world.data?.source]) {
      if (source) record(source.platform, source.project_id, source.version_id);
    }
    return {
      has: (platform, projectId) => projects.has(`${platform}:${projectId}`),
      hasVersion: (platform, projectId, versionId) => versions.has(`${platform}:${projectId}:${versionId}`),
    };
  }, [enabled, installed.data, pack.data, world.data]);
}
