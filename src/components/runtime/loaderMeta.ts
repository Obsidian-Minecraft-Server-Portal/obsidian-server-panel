import type { LoaderKind } from "../../api/loaders";

export interface LoaderMeta {
  label: string;
  icon: string;
  blurb: string;
}

export const LOADER_META: Record<LoaderKind, LoaderMeta> = {
  vanilla: { label: "Vanilla", icon: "lucide:box", blurb: "Official Mojang server" },
  fabric: { label: "Fabric", icon: "lucide:feather", blurb: "Lightweight mod loader" },
  quilt: { label: "Quilt", icon: "lucide:layers", blurb: "Fabric-compatible fork" },
  forge: { label: "Forge", icon: "lucide:hammer", blurb: "Classic large mod ecosystem" },
  neo_forge: { label: "NeoForge", icon: "lucide:anvil", blurb: "Modern Forge successor" },
  paper: { label: "Paper", icon: "lucide:scroll", blurb: "High-performance plugins" },
  purpur: { label: "Purpur", icon: "lucide:sparkles", blurb: "Paper fork with extras" },
  spigot: { label: "Spigot", icon: "lucide:puzzle", blurb: "Built with BuildTools" },
  craft_bukkit: { label: "CraftBukkit", icon: "lucide:package", blurb: "Built with BuildTools" },
  custom: { label: "Custom jar", icon: "lucide:upload", blurb: "Bring your own server.jar" },
};
