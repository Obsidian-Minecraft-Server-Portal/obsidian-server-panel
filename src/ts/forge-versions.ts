import $ from "./http.ts";

export type ForgeVersionList = {
    [key: string]: string[];
}

export async function getForgeVersions(): Promise<ForgeVersionList>
{
    return $.get("/api/forge/versions");
}