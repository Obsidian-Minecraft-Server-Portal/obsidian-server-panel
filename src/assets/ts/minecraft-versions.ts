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
export async function getMinecraftVersionDownloadUrl(version: string): Promise<string>
{
    const versions = await getMinecraftVersions();
    const versionData = versions.versions.find(v => v.id === version);
    if (!versionData) {
        throw new Error(`Version ${version} not found`);
    }
    const versionManifest = await $.get(versionData.url);
    if (!versionManifest.downloads || !versionManifest.downloads.server) {
        throw new Error(`Download URL for version ${version} not found`);
    }
    return versionManifest.downloads.server.url;
}