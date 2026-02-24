import {useParams, useSearchParams} from "react-router-dom";
import ErrorPage from "./ErrorPage.tsx";
import {ErrorBoundary} from "../components/ErrorBoundry.tsx";
import {useCallback, useEffect, useState} from "react";
import {Button, Card, CardHeader, Chip, Image, Link, Skeleton, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useServer} from "../providers/ServerProvider.tsx";
import {useMessage} from "../providers/MessageProvider.tsx";
import {MessageResponseType} from "../components/MessageModal.tsx";
import {ModDescription} from "../components/discover/ModDescription.tsx";
import {ModChangelog} from "../components/discover/ModChangelog.tsx";
import {ModVersions} from "../components/discover/ModVersions.tsx";
import type {ChangelogEntry, ModDetails, ModVersion} from "../types/ModTypes.tsx";
import "../ts/string-ext.ts";

export function ContentPage()
{
    const {type, platform, modId} = useParams();
    const [searchParams] = useSearchParams();
    const {loadServer, server, installMod} = useServer();
    const {open} = useMessage();

    const [modDetails, setModDetails] = useState<ModDetails | null>(null);
    const [modVersions, setModVersions] = useState<ModVersion[]>([]);
    const [changelog, setChangelog] = useState<ChangelogEntry[]>([]);
    const [loading, setLoading] = useState(true);
    const [versionsLoading, setVersionsLoading] = useState(true);
    const [selectedTab, setSelectedTab] = useState("description");
    const [changelogPage, setChangelogPage] = useState(1);

    if (platform !== "curseforge" && platform !== "modrinth")
    {
        console.error("Invalid platform specified in URL:", platform);
        return <ErrorPage code={500} message={`Invalid platform specified for Discover page ${platform}`}/>;
    }
    if (type !== "mods" && type !== "resourcepacks" && type !== "datapacks" && type !== "worlds" && type !== "packs")
    {
        console.error("Invalid type specified in URL:", type);
        return <ErrorPage code={500} message={`Invalid type specified for Discover page ${type}`}/>;
    }

    const fetchModrinthProject = async (projectId: string) =>
    {
        try
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
            } as ModDetails;
        } catch (error)
        {
            console.error("Failed to fetch Modrinth project:", error);
            throw error;
        }
    };

    const fetchCurseForgeProject = async (projectId: string) =>
    {
        try
        {
            const API_KEY = "$2a$10$qD2UJdpHaeDaQyGGaGS0QeoDnKq2EC7sX6YSjOxYHtDZSQRg04BCG";
            const response = await fetch(`https://api.curseforge.com/v1/mods/${projectId}`, {
                headers: {
                    "x-api-key": API_KEY
                }
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
            } as ModDetails;
        } catch (error)
        {
            console.error("Failed to fetch CurseForge project:", error);
            throw error;
        }
    };


    useEffect(() =>
    {
        const serverId = searchParams.get("sid");
        if (!serverId) return;
        loadServer(serverId);
    }, [searchParams]);

    const downloadModVersion = useCallback(async (version: ModVersion) =>
    {
        console.log("Downloading version:", version, "For server", server);
        if (!server)
        {
            await open({
                title: "No Server Selected",
                body: "Please select a server to download this mod to.",
                responseType: MessageResponseType.Close,
                severity: "warning"
            });
            return;
        }

        // Check compatibility
        const serverVersion = server.minecraft_version;
        const serverLoader = server.server_type;

        let compatible = true;
        let warnings = [];

        if (serverVersion && !version.game_versions.includes(serverVersion))
        {
            compatible = false;
            warnings.push(`This mod version supports Minecraft ${version.game_versions.join(", ")} but your server runs ${serverVersion}`);
        }

        if (serverLoader && !serverLoader.equalsIgnoreCase("vanilla") && !version.loaders.map(i => i.toLowerCase()).includes(serverLoader.toLowerCase()))
        {
            compatible = false;
            warnings.push(`This mod version supports ${version.loaders.join(", ")} but your server uses ${serverLoader}`);
        }

        if (!compatible)
        {
            const proceed = await open({
                title: "Compatibility Warning",
                body: `This mod version may not be compatible with your server:\n\n${warnings.join("\n")}\n\nDo you want to download it anyway?`,
                responseType: MessageResponseType.YesNo,
                severity: "warning"
            });

            if (!proceed) return;
        }

        try
        {
            const primaryFile = version.files.find(f => f.primary) || version.files[0];
            if (!primaryFile) throw new Error("No download file found");

            await installMod({
                downloadUrl: primaryFile.url,
                filename: primaryFile.filename,
                icon: modDetails?.icon_url,
                version: version.version_number,
                curseforgeId: platform === "curseforge" ? modDetails?.id : undefined,
                modrinthId: platform === "modrinth" ? modDetails?.id : undefined
            });


        } catch (error)
        {
            console.error("Failed to download mod:", error);
            await open({
                title: "Download Failed",
                body: "Failed to download the mod. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    }, [modDetails, server, open]);

    useEffect(() =>
    {
        if (!modId) return;

        const fetchData = async () =>
        {
            try
            {
                setLoading(true);
                let details: ModDetails;

                if (platform === "modrinth")
                {
                    details = await fetchModrinthProject(modId);
                } else
                {
                    details = await fetchCurseForgeProject(modId);
                }

                setModDetails(details);
            } catch (error)
            {
                console.error("Failed to fetch mod details:", error);
            } finally
            {
                setLoading(false);
            }
        };

        fetchData();
    }, [modId, platform]);

    useEffect(() =>
    {
        if (!modId || !modDetails) return;

        const fetchVersions = async () =>
        {
            try
            {
                setVersionsLoading(true);
                let versions: ModVersion[];

                if (platform === "modrinth")
                {
                    versions = await fetchModrinthVersions(modId);
                } else
                {
                    versions = await fetchCurseForgeVersions(modId);
                }

                setModVersions(versions);

                // Build changelog from versions
                const changelogEntries = versions
                    .filter(v => v.changelog && v.changelog.trim())
                    .map(v => ({
                        version: v.version_number,
                        version_type: v.version_type,
                        date: v.date_published,
                        changes: v.changelog || ""
                    }))
                    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());

                setChangelog(changelogEntries);
            } catch (error)
            {
                console.error("Failed to fetch mod versions:", error);
            } finally
            {
                setVersionsLoading(false);
            }
        };

        fetchVersions();
    }, [modId, platform, modDetails]);

    const formatDate = (dateString: string) =>
    {
        return new Date(dateString).toLocaleDateString();
    };

    const formatDownloads = (count: number) =>
    {
        if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
        if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
        return count.toString();
    };

    if (loading)
    {
        return (
            <div className="p-8 w-full mx-auto">
                <div className="flex gap-6 mb-6">
                    <Skeleton className="w-32 h-32 rounded-lg"/>
                    <div className="flex-1 space-y-4">
                        <Skeleton className="w-3/4 h-8"/>
                        <Skeleton className="w-full h-20"/>
                        <div className="flex gap-4">
                            <Skeleton className="w-24 h-6"/>
                            <Skeleton className="w-24 h-6"/>
                            <Skeleton className="w-24 h-6"/>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    if (!modDetails)
    {
        return <ErrorPage code={404} message="Mod not found"/>;
    }

    return (
        <ErrorBoundary>
            <div className="p-8 w-full mx-auto font-minecraft-body">
                {/* Header */}
                <Card className="mb-6" radius="none">
                    <CardHeader className="flex gap-6 p-6">
                        <Image
                            src={modDetails.icon_url || "/favicon.ico"}
                            alt={modDetails.name}
                            width={128}
                            height={128}
                            className="rounded-lg flex-shrink-0"
                            radius="sm"
                        />
                        <div className="flex-1 space-y-3">
                            <div className="flex items-center gap-3">
                                <h1 className="text-3xl font-minecraft-header font-bold">{modDetails.name}</h1>
                                <Chip
                                    color={platform === "modrinth" ? "success" : "warning"}
                                    variant="flat"
                                    className="text-xs"
                                >
                                    {platform === "modrinth" ? "Modrinth" : "CurseForge"}
                                </Chip>
                                <div className="flex gap-2 ml-auto">
                                    {/* Back to Search Button */}
                                    <Button
                                        as={Link}
                                        href={(() =>
                                        {
                                            const backUrl = searchParams.get("back");
                                            if (backUrl)
                                            {
                                                return decodeURIComponent(backUrl);
                                            }
                                            // Default back to server content tab if no back URL
                                            return server?.id ? `/app/servers/${server.id}?tab=content` : "/app";
                                        })()}
                                        radius="none"
                                        isIconOnly
                                    >
                                        <Icon icon="pixelarticons:arrow-left"/>
                                    </Button>

                                    {/* Open on Platform Button */}
                                    <Button
                                        as={Link}
                                        href={platform === "modrinth"
                                            ? `https://modrinth.com/mod/${modId}`
                                            : `https://www.curseforge.com/minecraft/mc-mods/${modDetails.slug || modId}`
                                        }
                                        target="_blank"
                                        radius="none"
                                        variant="solid"
                                        color={platform === "modrinth" ? "success" : "warning"}
                                        endContent={<Icon icon="pixelarticons:external-link"/>}
                                        className={
                                            platform === "modrinth"
                                                ? "text-black bg-[#1bd96a]"
                                                : "text-white bg-[#f16436]"
                                        }
                                    >
                                        Open on {platform === "modrinth" ? "Modrinth" : "CurseForge"}
                                    </Button>
                                </div>
                            </div>
                            <p className="text-default-600 text-lg">{modDetails.description}</p>
                            <div className="flex flex-wrap gap-4 text-sm">
                                <div className="flex items-center gap-1">
                                    <Icon icon="pixelarticons:download"/>
                                    <span className="font-semibold">{formatDownloads(modDetails.downloads)}</span>
                                    <span className="text-default-500">downloads</span>
                                </div>
                                {modDetails.followers && (
                                    <div className="flex items-center gap-1">
                                        <Icon icon="pixelarticons:heart"/>
                                        <span className="font-semibold">{formatDownloads(modDetails.followers)}</span>
                                        <span className="text-default-500">followers</span>
                                    </div>
                                )}
                                <div className="flex items-center gap-1">
                                    <Icon icon="pixelarticons:calendar"/>
                                    <span className="text-default-500">Updated {formatDate(modDetails.updated)}</span>
                                </div>
                            </div>
                            <div className="flex flex-wrap gap-2">
                                {modDetails.categories.slice(0, 6).map(category => (
                                    <Chip key={category} size="sm" variant="flat" color="primary">
                                        {category}
                                    </Chip>
                                ))}
                            </div>
                        </div>
                    </CardHeader>
                </Card>

                {/* Content Tabs */}
                <Tabs
                    selectedKey={selectedTab}
                    onSelectionChange={(key) => setSelectedTab(key as string)}
                    className="w-full"
                    radius="none"
                >
                    <Tab key="description" title="Description">
                        <ModDescription modDetails={modDetails}/>
                    </Tab>

                    <Tab key="changelog" title="Changelog">
                        <ModChangelog
                            changelog={changelog}
                            changelogPage={changelogPage}
                            onLoadMore={() => setChangelogPage(prev => prev + 1)}
                        />
                    </Tab>

                    <Tab key="versions" title="Versions">
                        <ModVersions
                            modVersions={modVersions}
                            versionsLoading={versionsLoading}
                            server={server || undefined}
                            serverId={server?.id || undefined}
                            onDownloadVersion={downloadModVersion}
                        />
                    </Tab>
                </Tabs>
            </div>
        </ErrorBoundary>
    );
}

export const fetchModrinthVersions = async (projectId: string) =>
{
    try
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
        })) as ModVersion[];
    } catch (error)
    {
        console.error("Failed to fetch Modrinth versions:", error);
        throw error;
    }
};

export const fetchCurseForgeVersions = async (projectId: string) =>
{
    try
    {
        const API_KEY = "$2a$10$qD2UJdpHaeDaQyGGaGS0QeoDnKq2EC7sX6YSjOxYHtDZSQRg04BCG";
        const response = await fetch(`https://api.curseforge.com/v1/mods/${projectId}/files`, {
            headers: {
                "x-api-key": API_KEY
            }
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
        })) as ModVersion[];
    } catch (error)
    {
        console.error("Failed to fetch CurseForge versions:", error);
        throw error;
    }
};