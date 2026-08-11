import $ from "./http.ts";

export type NeoForgeVersionList = {
    isSnapshot: boolean,
    versions: string[]
}

export async function getNeoForgeVersions(): Promise<NeoForgeVersionList>
{
    return $.get("/api/neoforge/versions");
}