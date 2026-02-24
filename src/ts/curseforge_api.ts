import $ from "jquery";

type CategoryResult = {
    data: Category[],
}
export type Category = {
    "id"?: number,
    "gameId"?: number,
    "name"?: string,
    "slug"?: string,
    "url"?: string,
    "iconUrl"?: string,
    "dateModified"?: string,
    "classId"?: number,
    "parentCategoryId"?: number,
    "displayIndex"?: number,
    "isClass"?: boolean,
}
type CurseforgeSearchResult = {
    data: ModDetails[],
    pagination: {
        index: number,
        pageSize: number,
        resultCount: number,
        totalCount: number,
    }
}
export type ModDetails = {
    id: number;
    gameId: number;
    name: string;
    slug: string;
    links: {
        websiteUrl: string;
        wikiUrl: string;
        issuesUrl: string;
        sourceUrl: string;
    };
    summary: string;
    status: number;
    downloadCount: number;
    isFeatured: boolean;
    primaryCategoryId: number;
    categories: {
        id: number;
        gameId: number;
        name: string;
        slug: string;
        url: string;
        iconUrl: string;
        dateModified: string;
        isClass: boolean;
        classId: number;
        parentCategoryId: number;
    }[];
    classId: number;
    authors: {
        id: number;
        name: string;
        url: string;
        avatarUrl: string;
    }[];
    logo: {
        id: number;
        modId: number;
        title: string;
        description: string;
        thumbnailUrl: string;
        url: string;
    };
    screenshots: {
        id: number;
        modId: number;
        title: string;
        description: string;
        thumbnailUrl: string;
        url: string;
    }[];
    mainFileId: number;
    latestFiles: {
        id: number;
        gameId: number;
        modId: number;
        isAvailable: boolean;
        displayName: string;
        fileName: string;
        releaseType: number;
        fileStatus: number;
        hashes: {
            value: string;
            algo: number;
        }[];
        fileDate: string;
        fileLength: number;
        downloadCount: number;
        downloadUrl: string;
        gameVersions: string[];
        sortableGameVersions: {
            gameVersionName: string;
            gameVersionPadded: string;
            gameVersion: string;
            gameVersionReleaseDate: string;
            gameVersionTypeId: number;
        }[];
        dependencies: {
            modId: number;
            relationType: number;
        }[];
        alternateFileId: number;
        isServerPack: boolean;
        fileFingerprint: number;
        modules: {
            name: string;
            fingerprint: number;
        }[];
    }[];
    latestFilesIndexes: {
        gameVersion: string;
        fileId: number;
        filename: string;
        releaseType: number;
        gameVersionTypeId: number;
        modLoader?: number;
    }[];
    dateCreated: string;
    dateModified: string;
    dateReleased: string;
    allowModDistribution: boolean;
    gamePopularityRank: number;
    isAvailable: boolean;
    thumbsUpCount: number;
};


export type SearchOptions = {
    query?: string,
    minecraftVersion?: string,
    loader?: string,
    category?: string,
    limit?: number,
    offset?: number,
}

export default class CurseForge
{
    private readonly base_url: string;

    constructor()
    {
        this.base_url = "/api/platform/curseforge";
    }

    async categories(): Promise<Category[]>
    {
        return (await $.ajax({
            "url": `${this.base_url}/categories`,
            "method": "GET",
            "timeout": 0,
        }) as CategoryResult).data;
    }

    async search(options: SearchOptions, abortSignal: AbortSignal): Promise<CurseforgeSearchResult>
    {
        const url = new URL(`${window.location.origin}${this.base_url}/search`);
        if (options.loader)
            url.searchParams.append(`modLoaderType`, options.loader);
        if (options.query)
            url.searchParams.append("query", options.query);
        if (options.minecraftVersion)
            url.searchParams.append("gameVersion", options.minecraftVersion);
        if (options.limit)
        {
            url.searchParams.append("pageSize", options.limit.toString());
            if (options.offset)
                url.searchParams.append("index", options.offset.toString());
        }

        if (options.category)
            url.searchParams.append("categoryId", options.category);

        return await fetch(url.toString(), {
            method: "GET",
            signal: abortSignal
        }).then(response =>
        {
            if (!response.ok)
            {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.json();
        });
    }

}