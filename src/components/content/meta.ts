import type { ContentKind, PlatformId, PlatformInfo, Project } from "../../api/content";
import type { LoaderKind } from "../../api/loaders";

export const PLATFORM_NAMES: Record<PlatformId, string> = {
  0: "CurseForge",
  1: "Modrinth",
  2: "Hangar",
  3: "Spiget",
};

export const PLATFORM_ICONS: Record<PlatformId, string> = {
  0: "simple-icons:curseforge",
  1: "simple-icons:modrinth",
  2: "lucide:paper-plane",
  3: "lucide:plug",
};

export const KIND_LABELS: Record<ContentKind, string> = {
  modpack: "Modpacks",
  mod: "Mods",
  plugin: "Plugins",
  datapack: "Data Packs",
  resourcepack: "Resource Packs",
  world: "Worlds",
};

export const KIND_ORDER: ContentKind[] = ["modpack", "mod", "plugin", "datapack", "resourcepack", "world"];

const FALLBACK_PLATFORMS: Record<ContentKind, PlatformId[]> = {
  modpack: [1, 0],
  mod: [1, 0],
  plugin: [1, 2, 0, 3],
  datapack: [1, 0],
  resourcepack: [1, 0],
  world: [0],
};

export function platformOffersKind(platforms: PlatformInfo[], platform: PlatformId, kind: ContentKind): boolean {
  const info = platforms.find((entry) => entry.id === platform);
  if (info?.available) return info.kinds.includes(kind);
  return FALLBACK_PLATFORMS[kind].includes(platform);
}

export const SOURCE_ORDER: PlatformId[] = [1, 0, 2, 3];

export function platformsForKind(platforms: PlatformInfo[], kind: ContentKind): PlatformId[] {
  const usable = SOURCE_ORDER.filter((id) =>
    platforms.some((entry) => entry.id === id && entry.available && entry.kinds.includes(kind)),
  );
  return usable.length ? usable : FALLBACK_PLATFORMS[kind];
}

const MOD_LOADERS: LoaderKind[] = ["fabric", "quilt", "forge", "neo_forge"];
const PLUGIN_LOADERS: LoaderKind[] = ["paper", "purpur", "spigot", "craft_bukkit"];

export const LOADERS_BY_KIND: Record<ContentKind, LoaderKind[]> = {
  modpack: [],
  mod: MOD_LOADERS,
  plugin: PLUGIN_LOADERS,
  datapack: [],
  resourcepack: [],
  world: [],
};

export function kindForLoader(loader: LoaderKind): ContentKind {
  return PLUGIN_LOADERS.includes(loader) ? "plugin" : "mod";
}

const downloadFormat = new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 });

export function formatDownloads(count: number): string {
  return downloadFormat.format(count);
}

export function projectKey(project: Project): string {
  return `${project.platform}:${project.id}`;
}
