import type {ModpackDetails, ModpackItemProps, ModpackVersion} from "../types/ModpackTypes.ts";

const CURSEFORGE_API_KEY = "$2a$10$qD2UJdpHaeDaQyGGaGS0QeoDnKq2EC7sX6YSjOxYHtDZSQRg04BCG";

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

        // Add facet for project type = modpack
        const facets = params.facets ? JSON.parse(params.facets) : [];
        facets.push(["project_type:modpack"]);
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
            slug: project.slug
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
        slug: data.slug
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
        files: version.files,
        changelog: version.changelog,
        dependencies: version.dependencies
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
        searchParams.set("gameId", "432"); // Minecraft
        searchParams.set("classId", "4471"); // Modpack class ID
        if (params.query) searchParams.set("searchFilter", params.query);
        if (params.categoryId) searchParams.set("categoryId", params.categoryId.toString());
        if (params.gameVersion) searchParams.set("gameVersion", params.gameVersion);
        if (params.modLoaderType) searchParams.set("modLoaderType", params.modLoaderType.toString());
        searchParams.set("pageSize", (params.limit || 20).toString());
        searchParams.set("index", (params.offset || 0).toString());

        const response = await fetch(`https://api.curseforge.com/v1/mods/search?${searchParams}`, {
            headers: {"x-api-key": CURSEFORGE_API_KEY}
        });
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
            slug: pack.slug
        }));
    } catch (error)
    {
        console.error("Failed to search CurseForge modpacks:", error);
        return [];
    }
}

export async function fetchCurseForgeModpackDetails(projectId: string): Promise<ModpackDetails>
{
    const response = await fetch(`https://api.curseforge.com/v1/mods/${projectId}`, {
        headers: {"x-api-key": CURSEFORGE_API_KEY}
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const result = await response.json();
    const data = result.data;

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
    const response = await fetch(`https://api.curseforge.com/v1/mods/${projectId}/files`, {
        headers: {"x-api-key": CURSEFORGE_API_KEY}
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const result = await response.json();

    return result.data.map((file: any) => ({
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
        changelog: file.changelog
    }));
}

// ============= ATLAUNCHER =============

const ATLAUNCHER_GRAPHQL_URL = "https://api.atlauncher.com/v2/graphql";

async function atlLauncherGraphQL(query: string, variables?: any): Promise<any>
{
    const response = await fetch(ATLAUNCHER_GRAPHQL_URL, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "User-Agent": "ObsidianServerPanel/1.0 (contact@example.com)" // Required by ATLauncher API
        },
        body: JSON.stringify({query, variables})
    });

    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const result = await response.json();

    if (result.errors)
    {
        console.error("GraphQL errors:", result.errors);
        throw new Error(result.errors[0]?.message || "GraphQL query failed");
    }

    return result.data;
}

export async function searchATLauncherModpacks(params: {
    query?: string;
}): Promise<ModpackItemProps[]>
{
    try
    {
        const gqlQuery = params.query
            ? `query SearchPacks($query: String!) {
                searchPacks(first: 50, query: $query, field: NAME) {
                    id
                    name
                    safeName
                    latestVersion {
                        minecraftVersion
                        updatedAt
                    }
                }
            }`
            : `query GetPacks {
                packs(first: 50) {
                    id
                    name
                    safeName
                    latestVersion {
                        minecraftVersion
                        updatedAt
                    }
                }
            }`;

        const variables = params.query ? {query: params.query} : undefined;
        const data = await atlLauncherGraphQL(gqlQuery, variables);

        const packs = params.query ? data.searchPacks : data.packs;

        return packs.map((pack: any) => ({
            packId: pack.safeName,
            platform: "atlauncher" as const,
            description: "",
            iconUrl: "https://atlauncher.com/assets/images/logo.svg",
            name: pack.name,
            downloadCount: 0,
            author: "Unknown",
            categories: [],
            lastUpdated: pack.latestVersion?.updatedAt ? new Date(pack.latestVersion.updatedAt) : new Date(),
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
    const gqlQuery = `query GetPack($safeName: String!) {
        pack(safeName: $safeName) {
            id
            name
            safeName
            versions {
                id
                version
                minecraftVersion
                changelog
                isRecommended
                createdAt
                updatedAt
                publishedAt
            }
        }
    }`;

    const data = await atlLauncherGraphQL(gqlQuery, {safeName: packId});
    const pack = data.pack;

    return {
        id: pack.id.toString(),
        name: pack.name,
        description: "",
        body: "",
        icon_url: "https://atlauncher.com/assets/images/logo.svg",
        downloads: 0,
        categories: [],
        versions: pack.versions?.map((v: any) => v.version) || [],
        game_versions: [...new Set<string>(pack.versions?.map((v: any) => v.minecraftVersion as string) || [])],
        loaders: [],
        published: pack.versions?.[0]?.createdAt || "",
        updated: pack.versions?.[0]?.updatedAt || "",
        authors: [],
        slug: pack.safeName
    };
}

export async function fetchATLauncherModpackVersions(packId: string): Promise<ModpackVersion[]>
{
    const gqlQuery = `query GetPackVersions($safeName: String!) {
        pack(safeName: $safeName) {
            safeName
            versions {
                id
                version
                minecraftVersion
                changelog
                isRecommended
                publishedAt
            }
        }
    }`;

    const data = await atlLauncherGraphQL(gqlQuery, {safeName: packId});
    const pack = data.pack;

    return pack.versions?.map((version: any) => ({
        id: version.id.toString(),
        version_number: version.version,
        name: version.version,
        version_type: version.isRecommended ? "release" : "beta",
        loaders: [],
        game_versions: [version.minecraftVersion],
        date_published: version.publishedAt || "",
        downloads: 0,
        files: [{
            url: "",
            filename: `${pack.safeName}-${version.version}.zip`,
            primary: true,
            size: 0
        }],
        changelog: version.changelog || ""
    })) || [];
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
        searchParams.set("build", "recommended");

        const response = await fetch(`https://api.technicpack.net/search?${searchParams}`);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        return data.modpacks?.slice(params.offset || 0, (params.offset || 0) + (params.limit || 20)).map((slug: string) => ({
            packId: slug,
            platform: "technic" as const,
            description: "",
            iconUrl: `https://solder.technicpack.net/packs/${slug}/icon.png`,
            name: slug.split("-").map(word => word.charAt(0).toUpperCase() + word.slice(1)).join(" "),
            downloadCount: 0,
            author: "Unknown",
            categories: [],
            lastUpdated: new Date(),
            slug: slug
        })) || [];
    } catch (error)
    {
        console.error("Failed to search Technic modpacks:", error);
        return [];
    }
}

export async function fetchTechnicModpackDetails(slug: string): Promise<ModpackDetails>
{
    const response = await fetch(`https://api.technicpack.net/modpack/${slug}?build=recommended`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();

    return {
        id: data.slug || slug,
        name: data.display_name || slug,
        description: data.description || "",
        body: data.description || "",
        icon_url: data.icon?.url,
        downloads: data.downloads || 0,
        categories: data.tags || [],
        versions: [data.recommended || ""],
        game_versions: [data.minecraft || ""],
        loaders: [],
        published: "",
        updated: "",
        authors: data.user ? [{name: data.user}] : [],
        slug: data.slug || slug
    };
}

export async function fetchTechnicModpackVersions(slug: string): Promise<ModpackVersion[]>
{
    const response = await fetch(`https://api.technicpack.net/modpack/${slug}?build=recommended`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();

    const versions: ModpackVersion[] = [];

    if (data.recommended)
    {
        versions.push({
            id: data.recommended,
            version_number: data.recommended,
            name: data.recommended,
            version_type: "release",
            loaders: [],
            game_versions: [data.minecraft || ""],
            date_published: "",
            downloads: 0,
            files: [{
                url: data.url || "",
                filename: `${slug}-${data.recommended}.zip`,
                primary: true,
                size: 0
            }],
            changelog: ""
        });
    }

    return versions;
}
