import $ from "jquery";

export type QuiltVersionList = {
    installer: QuiltInstallerVersion[]
    loader: QuiltLoaderVersion[],
}

export type QuiltInstallerVersion = {
    url: string,
    maven: string,
    version: string,
    stable: boolean
}

export type QuiltLoaderVersion = {
    separator: string,
    build: number,
    maven: string,
    version: string,
    stable: boolean,
}


export async function getQuiltVersions(): Promise<QuiltVersionList>
{
    return $.get("https://meta.quiltmc.org/v3/versions");
}

export const getQuiltServerUrl = (loaderVersion: string, minecraftVersion: string, installerVersion: string) =>
    `https://meta.quiltmc.org/v3/versions/loader/${minecraftVersion}/${loaderVersion}/${installerVersion}/server/jar`;
