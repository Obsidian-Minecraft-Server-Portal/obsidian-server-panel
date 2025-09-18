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
    private readonly api_key: string;
    private readonly base_url: string;

    constructor(api_key: string)
    {
        this.api_key = api_key;
        this.base_url = "https://api.curseforge.com/v1";
    }

    static getWithDefaultAPI(): CurseForge
    {
        const key: string = `$2a$10$qD2UJdpHaeDaQyGGaGS0QeoDnKq2EC7sX6YSjOxYHtDZSQRg04BCG`;
        return new CurseForge(key);
    }

    async categories(): Promise<Category[]>
    {
        return (await $.ajax({
            "url": `${this.base_url}/categories?gameId=432`,
            "method": "GET",
            "timeout": 0,
            "headers": {
                "x-api-key": this.api_key
            }
        }) as CategoryResult).data;
    }

    async search(options: SearchOptions, abortSignal: AbortSignal): Promise<CurseforgeSearchResult>
    {
        const url = new URL(`${this.base_url}/mods/search?gameId=432&classId=6&sortOrder=desc`);
        if (options.loader)
            url.searchParams.append(`modLoaderType`, options.loader);
        if (options.query)
            url.searchParams.append("searchFilter", options.query);
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
            headers: {
                "x-api-key": this.api_key
            },
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