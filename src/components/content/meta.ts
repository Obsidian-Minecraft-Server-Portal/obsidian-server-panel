import type { ContentKind, PlatformId, Project } from "../../api/content";
import type { LoaderKind } from "../../api/loaders";

export const PLATFORM_NAMES: Record<PlatformId, string> = {
  0: "CurseForge",
  1: "Modrinth",
  2: "Hangar",
  3: "Spiget",
};

export const KIND_LABELS: Record<ContentKind, string> = {
  modpack: "Modpacks",
  mod: "Mods",
  plugin: "Plugins",
};

export const KIND_ORDER: ContentKind[] = ["modpack", "mod", "plugin"];

export const PLATFORMS_BY_KIND: Record<ContentKind, PlatformId[]> = {
  modpack: [1, 0],
  mod: [1, 0],
  plugin: [1, 2, 0, 3],
};

const MOD_LOADERS: LoaderKind[] = ["fabric", "quilt", "forge", "neo_forge"];
const PLUGIN_LOADERS: LoaderKind[] = ["paper", "purpur", "spigot", "craft_bukkit"];

export const LOADERS_BY_KIND: Record<ContentKind, LoaderKind[]> = {
  modpack: [],
  mod: MOD_LOADERS,
  plugin: PLUGIN_LOADERS,
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
