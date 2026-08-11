import type {ModpackDetails, ModpackItemProps, ModpackPlatform, ModpackVersion} from "../types/ModpackTypes.ts";


// ============= MODRINTH =============

export async function searchModrinthModpacks(params: {
    query?: string;
    facets?: string;
    limit?: number;
    offset?: number;
}): Promise<ModpackItemProps[]>
{
    try
    {
        const searchParams = new URLSearchParams();
        if (params.query) searchParams.set("query", params.query);
        if (params.facets) searchParams.set("facets", params.facets);
        searchParams.set("limit", (params.limit || 20).toString());
        searchParams.set("offset", (params.offset || 0).toString());
        searchParams.set("index", "relevance");

        // Add facets for project type = modpack, restricted to server-compatible packs
        const facets = params.facets ? JSON.parse(params.facets) : [];
        facets.push(["project_type:modpack"]);
        facets.push(["server_side:required", "server_side:optional"]);
        searchParams.set("facets", JSON.stringify(facets));

        const response = await fetch(`/api/platform/modrinth/search?${searchParams}`);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        return data.hits.map((project: any) => ({
            packId: project.project_id,
            platform: "modrinth" as const,
            description: project.description,
            iconUrl: project.icon_url,
            name: project.title,
            downloadCount: project.downloads,
            author: project.author,
            categories: project.categories,
            lastUpdated: new Date(project.date_modified),
            slug: project.slug,
            serverPackAvailable: true
        }));
    } catch (error)
    {
        console.error("Failed to search Modrinth modpacks:", error);
        return [];
    }
}

export async function fetchModrinthModpackDetails(projectId: string): Promise<ModpackDetails>
{
    const response = await fetch(`/api/platform/modrinth/project/${projectId}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();

    return {
        id: data.id,
        name: data.title,
        description: data.description,
        body: data.body,
        icon_url: data.icon_url,
        downloads: data.downloads,
        followers: data.followers,
        categories: data.categories,
        license: data.license?.id,
        source_url: data.source_url,
        issues_url: data.issues_url,
        wiki_url: data.wiki_url,
        discord_url: data.discord_url,
        donation_urls: data.donation_urls,
        versions: data.versions,
        game_versions: data.game_versions,
        loaders: data.loaders,
        published: data.published,
        updated: data.updated,
        author: data.team,
        slug: data.slug,
        server_side: data.server_side,
        client_side: data.client_side
    };
}

export async function fetchModrinthModpackVersions(projectId: string): Promise<ModpackVersion[]>
{
    const response = await fetch(`/api/platform/modrinth/project/${projectId}/versions`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();

    return data.map((version: any) => ({
        id: version.id,
        version_number: version.version_number,
        name: version.name,
        version_type: version.version_type || "unknown",
        loaders: version.loaders,
        game_versions: version.game_versions,
        date_published: version.date_published,
        downloads: version.downloads,
        files: [...version.files].sort((a: any, b: any) => Number(b.primary) - Number(a.primary)),
        changelog: version.changelog,
        dependencies: version.dependencies,
        serverInstallable: true
    }));
}

// ============= CURSEFORGE =============

export async function searchCurseForgeModpacks(params: {
    query?: string;
    categoryId?: number;
    gameVersion?: string;
    modLoaderType?: number;
    limit?: number;
    offset?: number;
}): Promise<ModpackItemProps[]>
{
    try
    {
        const searchParams = new URLSearchParams();
        if (params.query) searchParams.set("query", params.query);
        if (params.categoryId) searchParams.set("categoryId", params.categoryId.toString());
        if (params.gameVersion) searchParams.set("gameVersion", params.gameVersion);
        if (params.modLoaderType) searchParams.set("modLoaderType", params.modLoaderType.toString());
        searchParams.set("pageSize", (params.limit || 20).toString());
        searchParams.set("index", (params.offset || 0).toString());

        const response = await fetch(`/api/platform/curseforge/search/modpacks?${searchParams}`);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const result = await response.json();

        return result.data.map((pack: any) => ({
            packId: pack.id.toString(),
            platform: "curseforge" as const,
            description: pack.summary,
            iconUrl: pack.logo?.url,
            name: pack.name,
            downloadCount: pack.downloadCount,
            author: pack.authors?.[0]?.name || "Unknown",
            categories: pack.categories?.map((cat: any) => cat.name) || [],
            lastUpdated: new Date(pack.dateModified),
            slug: pack.slug,
            serverPackAvailable: pack.latestFiles?.some((file: any) => file.serverPackFileId || file.isServerPack) ?? false
        }));
    } catch (error)
    {
        console.error("Failed to search CurseForge modpacks:", error);
        return [];
    }
}

export async function fetchCurseForgeModpackDetails(projectId: string): Promise<ModpackDetails>
{
    const response = await fetch(`/api/platform/curseforge/mod/${projectId}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();

    return {
        id: data.id.toString(),
        name: data.name,
        description: data.summary,
        body: data.description,
        icon_url: data.logo?.url,
        downloads: data.downloadCount,
        categories: data.categories?.map((cat: any) => cat.name) || [],
        license: data.license?.name,
        source_url: data.links?.sourceUrl,
        issues_url: data.links?.issuesUrl,
        wiki_url: data.links?.wikiUrl,
        versions: [],
        game_versions: data.latestFilesIndexes?.map((index: any) => index.gameVersion) || [],
        loaders: data.latestFilesIndexes?.map((index: any) => index.modLoader) || [],
        published: data.dateCreated,
        updated: data.dateModified,
        authors: data.authors?.map((author: any) => ({name: author.name, url: author.url})),
        slug: data.slug
    };
}

export async function fetchCurseForgeModpackVersions(projectId: string): Promise<ModpackVersion[]>
{
    const response = await fetch(`/api/platform/curseforge/mod/${projectId}/files`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const files = await response.json();

    return files.map((file: any) => ({
        id: file.id.toString(),
        version_number: file.displayName,
        name: file.fileName,
        version_type: file.releaseType === 1 ? "release" : file.releaseType === 2 ? "beta" : file.releaseType === 3 ? "alpha" : "unknown",
        loaders: file.gameVersions?.filter((v: string) => ["forge", "fabric", "quilt", "neoforge"].includes(v.toLowerCase())) || [],
        game_versions: file.gameVersions?.filter((v: string) => /^\d+\.\d+/.test(v)) || [],
        date_published: file.fileDate,
        downloads: file.downloadCount,
        files: [{
            hashes: {sha1: file.hashes?.[0]?.value || "", sha512: file.hashes?.[1]?.value || ""},
            url: file.downloadUrl,
            filename: file.fileName,
            primary: true,
            size: file.fileLength
        }],
        changelog: file.changelog,
        serverInstallable: !!(file.serverPackFileId || file.isServerPack),
        serverPackFileId: file.serverPackFileId ?? undefined
    }));
}

/**
 * Resolves the dedicated server pack file for a CurseForge modpack version.
 * Server installs MUST use this file, never the client zip.
 */
export async function fetchCurseForgeServerPackFile(modId: string, fileId: string): Promise<ModpackVersion["files"][0] | null>
{
    const response = await fetch(`/api/platform/curseforge/mod/${modId}/files/${fileId}/server-pack`);
    if (response.status === 404) return null;
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const file = await response.json();

    if (!file.downloadUrl) return null;
    return {
        hashes: {sha1: file.hashes?.find((h: any) => h.algo === 1)?.value || ""},
        url: file.downloadUrl,
        filename: file.fileName,
        primary: true,
        size: file.fileLength
    };
}

// ============= ATLAUNCHER =============

const ATLAUNCHER_ICON = "https://atlauncher.com/assets/images/logo.svg";

export async function searchATLauncherModpacks(params: {
    query?: string;
}): Promise<ModpackItemProps[]>
{
    try
    {
        const searchParams = new URLSearchParams();
        if (params.query) searchParams.set("query", params.query);

        const response = await fetch(`/api/platform/atlauncher/search?${searchParams}`);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const packs = await response.json();

        return (packs || []).map((pack: any) => ({
            packId: pack.safeName,
            platform: "atlauncher" as const,
            description: pack.description || "",
            iconUrl: ATLAUNCHER_ICON,
            name: pack.name,
            downloadCount: 0,
            author: "ATLauncher",
            categories: pack.latestVersion?.minecraftVersion ? [pack.latestVersion.minecraftVersion] : [],
            lastUpdated: pack.latestVersion?.publishedAt ? new Date(pack.latestVersion.publishedAt) : new Date(),
            slug: pack.safeName
        }));
    } catch (error)
    {
        console.error("Failed to search ATLauncher modpacks:", error);
        return [];
    }
}

export async function fetchATLauncherModpackDetails(packId: string): Promise<ModpackDetails>
{
    const response = await fetch(`/api/platform/atlauncher/pack/${packId}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const pack = await response.json();

    const versions = pack.versions || [];
    return {
        id: pack.id.toString(),
        name: pack.name,
        description: pack.description || "",
        body: pack.description || "",
        icon_url: ATLAUNCHER_ICON,
        downloads: 0,
        categories: [],
        versions: versions.map((v: any) => v.version),
        game_versions: [...new Set<string>(versions.map((v: any) => v.minecraft as string))],
        loaders: [],
        published: versions.length ? new Date(versions[versions.length - 1].published * 1000).toISOString() : "",
        updated: versions.length ? new Date(versions[0].published * 1000).toISOString() : "",
        authors: [],
        slug: pack.safeName
    };
}

export async function fetchATLauncherModpackVersions(packId: string): Promise<ModpackVersion[]>
{
    const response = await fetch(`/api/platform/atlauncher/pack/${packId}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const pack = await response.json();

    return (pack.versions || []).map((version: any, index: number) => ({
        id: version.version,
        version_number: version.version,
        name: version.version,
        version_type: index === 0 ? "release" : "unknown",
        loaders: [],
        game_versions: [version.minecraft],
        date_published: version.published ? new Date(version.published * 1000).toISOString() : "",
        downloads: 0,
        files: [{
            url: "",
            filename: `${pack.safeName}-${version.version}.zip`,
            primary: true,
            size: 0
        }],
        changelog: ""
    }));
}

// ============= TECHNIC =============

export async function searchTechnicModpacks(params: {
    query?: string;
    limit?: number;
    offset?: number;
}): Promise<ModpackItemProps[]>
{
    try
    {
        const searchParams = new URLSearchParams();
        if (params.query) searchParams.set("q", params.query);

        const response = await fetch(`/api/platform/technic/search?${searchParams}`);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        return (data.modpacks || []).slice(params.offset || 0, (params.offset || 0) + (params.limit || 20)).map((pack: any) => ({
            packId: pack.slug,
            platform: "technic" as const,
            description: "",
            iconUrl: pack.iconUrl,
            name: pack.name,
            downloadCount: 0,
            author: "Unknown",
            categories: [],
            lastUpdated: new Date(),
            slug: pack.slug
        }));
    } catch (error)
    {
        console.error("Failed to search Technic modpacks:", error);
        return [];
    }
}

export async function fetchTechnicModpackDetails(slug: string): Promise<ModpackDetails>
{
    const response = await fetch(`/api/platform/technic/modpack/${slug}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();

    return {
        id: data.name || slug,
        name: data.displayName || slug,
        description: data.description || "",
        body: data.description || "",
        icon_url: data.icon?.url,
        downloads: data.installs || 0,
        categories: data.tags || [],
        versions: [data.version || ""],
        game_versions: [data.minecraft || ""],
        loaders: [],
        published: "",
        updated: "",
        authors: data.user ? [{name: data.user}] : [],
        slug: data.name || slug
    };
}

export async function fetchTechnicModpackVersions(slug: string): Promise<ModpackVersion[]>
{
    const response = await fetch(`/api/platform/technic/modpack/${slug}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();

    if (!data.version) return [];

    return [{
        id: data.version,
        version_number: data.version,
        name: data.version,
        version_type: "release",
        loaders: [],
        game_versions: [data.minecraft || ""],
        date_published: "",
        downloads: 0,
        files: [{
            url: data.serverPackUrl || "",
            filename: `${slug}-${data.version}.zip`,
            primary: true,
            size: 0
        }],
        changelog: ""
    }];
}

// ============= INSTALLATION =============

export async function installModpackServer(params: {
    platform: ModpackPlatform;
    packId: string;
    version: string;
    name: string;
    javaExecutable: string;
}): Promise<string>
{
    const response = await fetch("/api/server/from-modpack", {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({
            platform: params.platform,
            pack_id: params.packId,
            version: params.version,
            name: params.name,
            java_executable: params.javaExecutable
        })
    });
    const data = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(data.error || `HTTP ${response.status}`);
    return data.server_id;
}
