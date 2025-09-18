import $ from "jquery";

export type NeoForgeVersionList = {
    isSnapshot: boolean,
    versions: string[]
}

export async function getNeoForgeVersions(): Promise<NeoForgeVersionList>
{
    return $.get("https://maven.neoforged.net/api/maven/versions/releases/net%2Fneoforged%2Fneoforge");
}