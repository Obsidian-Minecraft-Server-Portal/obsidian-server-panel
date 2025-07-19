import $ from "jquery";

export type MinecraftVersionList = {
    latest: {
        release: string;
        snapshot: string;
    };
    versions: MinecraftVersion[];
}

export type MinecraftVersion = {
    id: string;
    type: "release" | "snapshot" | "old_beta" | "old_alpha";
    url: string;
    time: string;
    releaseTime: string;
}

export async function getMinecraftVersions(): Promise<MinecraftVersionList>
{
    return $.get("https://launchermeta.mojang.com/mc/game/version_manifest.json");
}