export type ModDetails = {
    id: string;
    name: string;
    description: string;
    body?: string;
    icon_url?: string;
    downloads: number;
    followers?: number;
    categories: string[];
    license?: string;
    source_url?: string;
    issues_url?: string;
    wiki_url?: string;
    discord_url?: string;
    donation_urls?: Array<{ id: string, platform: string, url: string }>;
    versions: string[];
    game_versions: string[];
    loaders: string[];
    published: string;
    updated: string;
    author?: string;
    authors?: Array<{ name: string, url?: string }>;
    slug?: string;
};

export type ModVersion = {
    id: string;
    version_number: string;
    name: string;
    version_type: "release" | "beta" | "alpha" | "unknown";
    loaders: string[];
    game_versions: string[];
    date_published: string;
    downloads: number;
    files: Array<{
        hashes: { sha1: string, sha512: string };
        url: string;
        filename: string;
        primary: boolean;
        size: number;
        file_type?: string;
    }>;
    changelog?: string;
    dependencies?: Array<{
        version_id?: string;
        project_id?: string;
        file_name?: string;
        dependency_type: "required" | "optional" | "incompatible" | "embedded";
    }>;
};

export type ChangelogEntry = {
    version: string;
    version_type: "release" | "beta" | "alpha" | "unknown";
    date: string;
    changes: string;
};

export type ServerInfo = {
    id: string;
    minecraft_version?: string;
    server_type?: string;
    [key: string]: any;
};
