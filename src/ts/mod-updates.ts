import {LoaderType} from "../providers/ServerProvider.tsx";

export interface ModUpdateInfo
{
    version: string;
    downloadUrl: string;
    filename: string;
    gameVersions: string[];
    loaders: string[];
}

export interface ModrinthVersion
{
    id: string;
    project_id: string;
    author_id: string;
    featured: boolean;
    name: string;
    version_number: string;
    changelog: string;
    date_published: string;
    downloads: number;
    version_type: string;
    status: string;
    requested_status: string;
    files: {
        hashes: {
            sha512: string;
            sha1: string;
        };
        url: string;
        filename: string;
        primary: boolean;
        size: number;
        file_type: string;
    }[];
    dependencies: any[];
    game_versions: string[];
    loaders: string[];
}

export interface CurseForgeFile
{
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
    dependencies: any[];
    alternateFileId: number;
    isServerPack: boolean;
    serverPackFileId: number;
    isEarlyAccessContent: boolean;
    earlyAccessEndDate: string;
    fileFingerprint: number;
    modules: {
        name: string;
        fingerprint: number;
    }[];
}

/**
 * Convert server loader type to Modrinth loader format
 */
function serverLoaderToModrinthLoader(serverType: LoaderType): string[]
{
    switch (serverType)
    {
        case "fabric":
            return ["fabric"];
        case "forge":
            return ["forge"];
        case "neoforge":
            return ["neoforge"];
        case "quilt":
            return ["quilt", "fabric"]; // Quilt is compatible with Fabric
        case "vanilla":
            return []; // No loader for vanilla
        default:
            return [];
    }
}

/**
 * Convert server loader type to CurseForge loader format
 */
function serverLoaderToCurseForgeLoader(serverType: LoaderType): string[]
{
    switch (serverType)
    {
        case "fabric":
            return ["Fabric"];
        case "forge":
            return ["Forge"];
        case "neoforge":
            return ["NeoForge"];
        case "quilt":
            return ["Quilt", "Fabric"]; // Quilt is compatible with Fabric
        case "vanilla":
            return []; // No loader for vanilla
        default:
            return [];
    }
}

/**
 * Check if a version is compatible with the server
 */
function isVersionCompatible(
    gameVersions: string[],
    loaders: string[],
    serverMinecraftVersion: string,
    serverLoaderType: LoaderType
): boolean
{
    // Check Minecraft version compatibility
    const isMinecraftVersionCompatible = gameVersions.includes(serverMinecraftVersion);

    // Check loader compatibility
    const compatibleLoaders = serverLoaderToModrinthLoader(serverLoaderType);
    const isLoaderCompatible = compatibleLoaders.length === 0 ||
        compatibleLoaders.some(loader => loaders.includes(loader));

    return isMinecraftVersionCompatible && isLoaderCompatible;
}

/**
 * Check for updates on Modrinth
 */
export async function checkModrinthUpdate(
    modrinthId: string,
    currentVersion: string,
    serverMinecraftVersion: string,
    serverLoaderType: LoaderType
): Promise<ModUpdateInfo | null>
{
    try
    {
        const response = await fetch(`/api/platform/modrinth/project/${modrinthId}/versions`);
        if (!response.ok)
        {
            console.warn(`Failed to fetch Modrinth versions for ${modrinthId}: ${response.status}`);
            return null;
        }

        const versions: ModrinthVersion[] = await response.json();

        // Filter versions that are compatible with the server
        const compatibleVersions = versions.filter(version =>
            isVersionCompatible(
                version.game_versions,
                version.loaders,
                serverMinecraftVersion,
                serverLoaderType
            )
        );

        if (compatibleVersions.length === 0)
        {
            return null;
        }

        // Sort by date (newest first)
        compatibleVersions.sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime());

        const latestVersions = compatibleVersions.filter(version => version.loaders.includes(serverLoaderType));
        const indexOfCurrentVersion = latestVersions.findIndex(version => version.version_number === currentVersion);
        if (indexOfCurrentVersion === -1 || indexOfCurrentVersion >= 1)
        {
            // If the current version is not found or there is a newer version
            const latestVersion = latestVersions[0];
            if (latestVersion.files.length > 0)
            {
                const file = latestVersion.files[0];
                return {
                    version: latestVersion.version_number,
                    downloadUrl: file.url,
                    filename: file.filename,
                    gameVersions: latestVersion.game_versions,
                    loaders: latestVersion.loaders
                };
            }
        }

        return null;
    } catch (error)
    {
        console.error(`Error checking Modrinth update for ${modrinthId}:`, error);
        return null;
    }
}

/**
 * Check for updates on CurseForge
 */
export async function checkCurseForgeUpdate(
    curseforgeId: string,
    currentVersion: string,
    serverMinecraftVersion: string,
    serverLoaderType: LoaderType
): Promise<ModUpdateInfo | null>
{
    try
    {
        const response = await fetch(`/api/platform/curseforge/mod/${curseforgeId}/files`);

        if (!response.ok)
        {
            console.warn(`Failed to fetch CurseForge files for ${curseforgeId}: ${response.status}`);
            return null;
        }

        const files: CurseForgeFile[] = await response.json();

        // Filter files that are compatible with the server
        const compatibleLoaders = serverLoaderToCurseForgeLoader(serverLoaderType);
        const compatibleFiles = files.filter(file =>
        {
            const isMinecraftVersionCompatible = file.gameVersions.includes(serverMinecraftVersion);
            const isLoaderCompatible = compatibleLoaders.length === 0 ||
                compatibleLoaders.some(loader => file.gameVersions.includes(loader));

            return isMinecraftVersionCompatible && isLoaderCompatible && file.isAvailable;
        });

        if (compatibleFiles.length === 0)
        {
            return null;
        }

        // Sort by date (newest first)
        compatibleFiles.sort((a, b) => new Date(b.fileDate).getTime() - new Date(a.fileDate).getTime());

        const latestFile = compatibleFiles[0];

        // Check if there's an update available (comparing by filename since CurseForge doesn't have version numbers)
        if (latestFile.fileName !== currentVersion && latestFile.downloadUrl)
        {
            return {
                version: latestFile.displayName,
                downloadUrl: latestFile.downloadUrl,
                filename: latestFile.fileName,
                gameVersions: latestFile.gameVersions,
                loaders: latestFile.gameVersions.filter(v => compatibleLoaders.includes(v))
            };
        }

        return null;
    } catch (error)
    {
        console.error(`Error checking CurseForge update for ${curseforgeId}:`, error);
        return null;
    }
}
