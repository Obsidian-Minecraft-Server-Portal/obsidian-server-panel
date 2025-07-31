export type ModrinthProject = {
    project_id: string;
    project_type: "mod" | "modpack" | "resourcepack" | "shader";
    slug: string;
    author: string;
    title: string;
    description: string;
    categories: string[];
    display_categories: string[];
    client_side: "required" | "optional" | "unsupported";
    server_side: "required" | "optional" | "unsupported";
    versions: string[];
    downloads: number;
    follows: number;
    icon_url?: string;
    date_created: string;
    date_modified: string;
    latest_version: string;
    license: string;
    gallery: string[];
    featured_gallery?: string;
    color?: number;
};

export type ModrinthSearchResult = {
    hits: ModrinthProject[];
    offset: number;
    limit: number;
    total_hits: number;
};

export type ModrinthCategory = {
    icon: string;
    name: string;
    project_type: "mod" | "modpack" | "resourcepack" | "shader";
    header: string;
};

export type ModrinthGameVersion = {
    version: string;
    version_type: "release" | "snapshot" | "alpha" | "beta";
    date: string;
    major: boolean;
};

export type ModrinthLoader = {
    icon: string;
    name: string;
    supported_project_types: ("mod" | "modpack" | "resourcepack" | "shader")[];
};

export type SearchOptions = {
    query?: string;
    facets?: string[][];
    index?: "relevance" | "downloads" | "follows" | "newest" | "updated";
    offset?: number;
    limit?: number;
};

export default class Modrinth {
    private readonly base_url: string;

    constructor() {
        this.base_url = "https://api.modrinth.com/v2";
    }

    static getInstance(): Modrinth {
        return new Modrinth();
    }

    async categories(): Promise<ModrinthCategory[]> {
        const response = await fetch(`${this.base_url}/tag/category`, {
            method: "GET",
            headers: {
                "User-Agent": "obsidian-server-panel/1.0.0"
            }
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        return response.json();
    }

    async gameVersions(): Promise<ModrinthGameVersion[]> {
        const response = await fetch(`${this.base_url}/tag/game_version`, {
            method: "GET",
            headers: {
                "User-Agent": "obsidian-server-panel/1.0.0"
            }
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        return response.json();
    }

    async loaders(): Promise<ModrinthLoader[]> {
        const response = await fetch(`${this.base_url}/tag/loader`, {
            method: "GET",
            headers: {
                "User-Agent": "obsidian-server-panel/1.0.0"
            }
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        return response.json();
    }

    async search(options: SearchOptions, abortSignal: AbortSignal): Promise<ModrinthSearchResult> {
        const url = new URL(`${this.base_url}/search`);

        if (options.query) {
            url.searchParams.append("query", options.query);
        }

        if (options.facets && options.facets.length > 0) {
            url.searchParams.append("facets", JSON.stringify(options.facets));
        }

        if (options.index) {
            url.searchParams.append("index", options.index);
        }

        if (options.offset !== undefined) {
            url.searchParams.append("offset", options.offset.toString());
        }

        if (options.limit !== undefined) {
            url.searchParams.append("limit", options.limit.toString());
        }

        const response = await fetch(url.toString(), {
            method: "GET",
            headers: {
                "User-Agent": "obsidian-server-panel/1.0.0"
            },
            signal: abortSignal
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        return response.json();
    }

    // Helper method to build facets for search
    static buildFacets(categories?: string[], gameVersions?: string[], loaders?: string[]): string[][] {
        const facets: string[][] = [];

        // Add project type filter for mods
        facets.push(["project_type:mod"]);

        if (categories && categories.length > 0) {
            facets.push(categories.map(cat => `categories:${cat}`));
        }

        if (gameVersions && gameVersions.length > 0) {
            facets.push(gameVersions.map(version => `versions:${version}`));
        }

        if (loaders && loaders.length > 0) {
            facets.push(loaders.map(loader => `categories:${loader}`));
        }

        return facets;
    }
}
