import $ from "jquery";

export type FabricVersionList = {
    installer: FabricInstallerVersion[]
    loader: FabricLoaderVersion[],
}

export type FabricInstallerVersion = {
    url: string,
    maven: string,
    version: string,
    stable: boolean
}

export type FabricLoaderVersion = {
    separator: string,
    build: number,
    maven: string,
    version: string,
    stable: boolean,
}


export async function getFabricVersions(): Promise<FabricVersionList>
{
    return $.get("https://meta.fabricmc.net/v2/versions/");
}

export const getFabricServerUrl = (loaderVersion: string, minecraftVersion: string, installerVersion: string) => `https://meta.fabricmc.net/v2/versions/loader/${minecraftVersion}/${loaderVersion}/${installerVersion}/server/jar`;