import {useParams, useSearchParams} from "react-router-dom";
import ErrorPage from "./ErrorPage.tsx";
import {ErrorBoundary} from "../components/ErrorBoundry.tsx";
import {useEffect, useState} from "react";
import {Button, Card, CardBody, CardHeader, Chip, Divider, Image, Input, Link, Select, SelectItem, Skeleton, Spinner, Tab, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useServer} from "../providers/ServerProvider.tsx";
import {useMessage} from "../providers/MessageProvider.tsx";
import {MessageResponseType} from "../components/MessageModal.tsx";
import ReactMarkdown from "react-markdown";
import {useInfiniteScroll} from "@heroui/use-infinite-scroll";
import {useAsyncList} from "@react-stately/data";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";

type ModDetails = {
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

type ModVersion = {
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

type ChangelogEntry = {
    version: string;
    version_type: "release" | "beta" | "alpha" | "unknown";
    date: string;
    changes: string;
};

export function Discover()
{
    const {type, platform, modId} = useParams();
    const [searchParams] = useSearchParams();
    const serverId = searchParams.get("sid");
    const {server} = useServer();
    const {open} = useMessage();

    const [modDetails, setModDetails] = useState<ModDetails | null>(null);
    const [modVersions, setModVersions] = useState<ModVersion[]>([]);
    const [changelog, setChangelog] = useState<ChangelogEntry[]>([]);
    const [loading, setLoading] = useState(true);
    const [versionsLoading, setVersionsLoading] = useState(true);
    const [selectedTab, setSelectedTab] = useState("description");
    const [versionFilter, setVersionFilter] = useState("");
    const [gameVersionFilter, setGameVersionFilter] = useState("");
    const [loaderFilter, setLoaderFilter] = useState("");
    const [typeFilter, setTypeFilter] = useState("");

    // Changelog pagination
    const [changelogPage, setChangelogPage] = useState(1);
    const CHANGELOG_PER_PAGE = 5;

    // Infinite scroll for versions
    const versionsPerPage = 50;
    const [hasMoreVersions, setHasMoreVersions] = useState(false);

    const versionsList = useAsyncList({
        // @ts-ignore
        async load({signal, cursor})
        {
            if (!modVersions.length)
            {
                setHasMoreVersions(false);
                return {items: [], cursor: null};
            }

            const startIndex = cursor ? parseInt(cursor) : 0;
            const endIndex = startIndex + versionsPerPage;

            // Apply filters to all versions first
            const filtered = modVersions.filter(version =>
            {
                return (
                    (!versionFilter || version.version_number.toLowerCase().includes(versionFilter.toLowerCase()) ||
                        version.name.toLowerCase().includes(versionFilter.toLowerCase())) &&
                    (!gameVersionFilter || version.game_versions.some(v => v.includes(gameVersionFilter))) &&
                    (!loaderFilter || version.loaders.some(l => l.toLowerCase().includes(loaderFilter.toLowerCase()))) &&
                    (!typeFilter || version.version_type === typeFilter)
                );
            });

            const pageItems = filtered.slice(startIndex, endIndex);
            const hasMore = endIndex < filtered.length;

            setHasMoreVersions(hasMore);

            return {
                items: pageItems,
                cursor: hasMore ? endIndex.toString() : null
            };
        }
    });

    const [loaderRef, scrollerRef] = useInfiniteScroll({
        hasMore: hasMoreVersions,
        onLoadMore: versionsList.loadMore
    });

    // Reload versions list when filters change
    useEffect(() =>
    {
        if (modVersions.length > 0)
        {
            versionsList.reload();
        }
    }, [versionFilter, gameVersionFilter, loaderFilter, typeFilter, modVersions]);

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
            const response = await fetch(`https://api.modrinth.com/v2/project/${projectId}`);
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

    const fetchModrinthVersions = async (projectId: string) =>
    {
        try
        {
            const response = await fetch(`https://api.modrinth.com/v2/project/${projectId}/version`);
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

    const fetchCurseForgeVersions = async (projectId: string) =>
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

    const downloadModVersion = async (version: ModVersion) =>
    {
        if (!server || !serverId)
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

        if (serverLoader && serverLoader !== "vanilla" && !version.loaders.includes(serverLoader))
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
            if (!primaryFile)
            {
                throw new Error("No download file found");
            }

            const response = await fetch(`/api/server/${serverId}/download-mod`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    download_url: primaryFile.url,
                    filename: primaryFile.filename
                })
            });

            if (!response.ok)
            {
                throw new Error(`HTTP ${response.status}`);
            }

            const result = await response.json();
            console.log("Download result:", result);

            await open({
                title: "Download Successful",
                body: `${modDetails?.name} v${version.version_number} has been downloaded to your server.`,
                responseType: MessageResponseType.Close,
                severity: "success"
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
    };

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

    const filteredChangelog = changelog.slice(0, changelogPage * CHANGELOG_PER_PAGE);

    // const filteredVersions = modVersions.filter(version =>
    // {
    //     return (
    //         (!versionFilter || version.version_number.toLowerCase().includes(versionFilter.toLowerCase()) ||
    //             version.name.toLowerCase().includes(versionFilter.toLowerCase())) &&
    //         (!gameVersionFilter || version.game_versions.some(v => v.includes(gameVersionFilter))) &&
    //         (!loaderFilter || version.loaders.some(l => l.toLowerCase().includes(loaderFilter.toLowerCase()))) &&
    //         (!typeFilter || version.version_type === typeFilter)
    //     );
    // });

    const getVersionTypeIcon = (type: string) =>
    {
        switch (type)
        {
            case "release":
                return "R";
            case "beta":
                return "B";
            case "alpha":
                return "A";
            default:
                return "?";
        }
    };

    const getVersionTypeColor = (type: string) =>
    {
        switch (type)
        {
            case "release":
                return "success";
            case "beta":
                return "warning";
            case "alpha":
                return "danger";
            default:
                return "default";
        }
    };

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

    // Custom markdown components for proper styling
    const markdownComponents = {
        h1: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h1 className="text-4xl font-bold mb-2">{children}</h1>
                <Divider />
            </div>
        ),
        h2: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h2 className="text-3xl font-bold mb-2">{children}</h2>
                <Divider />
            </div>
        ),
        h3: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h3 className="text-2xl font-bold mb-2">{children}</h3>
                <Divider />
            </div>
        ),
        h4: ({children}: any) => (
            <div className="mb-4 mt-8">
                <h4 className="text-xl font-bold mb-2">{children}</h4>
                <Divider />
            </div>
        ),
        ul: ({children}: any) => (
            <ul className="list-disc ml-8 my-4" style={{listStyleType: 'disc'}}>
                {children}
            </ul>
        ),
        img: ({src, alt}: any) => (
            <Image
                src={src}
                alt={alt || ""}
                radius="none"
                className="my-4"
            />
        )
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
                                            return serverId ? `/app/servers/${serverId}?tab=content` : "/app";
                                        })()}
                                        radius="none"
                                        variant="ghost"
                                        color="primary"
                                        startContent={<Icon icon="pixelarticons:arrow-left"/>}
                                    >
                                        Back to Search
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
                                        variant="ghost"
                                        color={platform === "modrinth" ? "success" : "warning"}
                                        endContent={<Icon icon="pixelarticons:external-link"/>}
                                        className={
                                            platform === "modrinth"
                                                ? "text-[#1bd96a] border-[#1bd96a] hover:bg-[#1bd96a] hover:text-black"
                                                : "text-[#f16436] border-[#f16436] hover:bg-[#f16436] hover:text-white"
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
                        <Card radius="none">
                            <CardBody className="p-6">
                                {modDetails.body ? (
                                    <div className="prose prose-sm max-w-none">
                                        <ReactMarkdown
                                            remarkPlugins={[remarkGfm]}
                                            rehypePlugins={[rehypeRaw, rehypeSanitize]}
                                            components={markdownComponents}
                                        >
                                            {modDetails.body}
                                        </ReactMarkdown>
                                    </div>
                                ) : (
                                    <p className="text-default-600">{modDetails.description}</p>
                                )}
                            </CardBody>
                        </Card>
                    </Tab>

                    <Tab key="changelog" title="Changelog">
                        <Card radius="none">
                            <CardBody className="p-6">
                                {changelog.length > 0 ? (
                                    <div className="space-y-6">
                                        {filteredChangelog.map((entry, index) => (
                                            <div key={index} className="border-l-4 border-primary pl-4">
                                                <div className="flex items-center gap-3 mb-2">
                                                    <Chip
                                                        size="sm"
                                                        color={getVersionTypeColor(entry.version_type) as any}
                                                        variant="flat"
                                                    >
                                                        {getVersionTypeIcon(entry.version_type)} {entry.version}
                                                    </Chip>
                                                    <span className="text-default-500 text-sm">
                                                        {formatDate(entry.date)}
                                                    </span>
                                                </div>
                                                <div className="prose prose-sm max-w-none">
                                                    <ReactMarkdown
                                                        remarkPlugins={[remarkGfm]}
                                                        rehypePlugins={[rehypeRaw, rehypeSanitize]}
                                                    >
                                                        {entry.changes}
                                                    </ReactMarkdown>
                                                </div>
                                            </div>
                                        ))}
                                        {changelog.length > CHANGELOG_PER_PAGE && (
                                            <div className="flex justify-center">
                                                <Button
                                                    variant="ghost"
                                                    color="primary"
                                                    onPress={() => setChangelogPage(prev => prev + 1)}
                                                    radius={"none"}
                                                >
                                                    Load more
                                                </Button>
                                            </div>
                                        )}
                                    </div>
                                ) : (
                                    <p className="text-default-500">No changelog available</p>
                                )}
                            </CardBody>
                        </Card>
                    </Tab>

                    <Tab key="versions" title="Versions">
                        <Card radius="none">
                            <CardBody className="p-6">
                                {/* Filters */}
                                <div className="flex gap-4 mb-6 flex-wrap">
                                    <Input
                                        placeholder="Search versions..."
                                        value={versionFilter}
                                        onValueChange={setVersionFilter}
                                        className="max-w-xs"
                                        radius="none"
                                        startContent={<Icon icon="pixelarticons:search"/>}
                                    />
                                    <Select
                                        placeholder="Game Version"
                                        selectedKeys={gameVersionFilter ? [gameVersionFilter] : []}
                                        onSelectionChange={(keys) => setGameVersionFilter([...keys][0] as string || "")}
                                        className="max-w-xs"
                                        radius="none"
                                    >
                                        {Array.from(new Set(modVersions.flatMap(v => v.game_versions))).map(version => (
                                            <SelectItem key={version} textValue={version}>{version}</SelectItem>
                                        ))}
                                    </Select>
                                    <Select
                                        placeholder="Loader"
                                        selectedKeys={loaderFilter ? [loaderFilter] : []}
                                        onSelectionChange={(keys) => setLoaderFilter([...keys][0] as string || "")}
                                        className="max-w-xs"
                                        radius="none"
                                    >
                                        {Array.from(new Set(modVersions.flatMap(v => v.loaders))).map(loader => (
                                            <SelectItem key={loader} textValue={loader}>{loader}</SelectItem>
                                        ))}
                                    </Select>
                                    <Select
                                        placeholder="Release Type"
                                        selectedKeys={typeFilter ? [typeFilter] : []}
                                        onSelectionChange={(keys) => setTypeFilter([...keys][0] as string || "")}
                                        className="max-w-xs"
                                        radius="none"
                                    >
                                        <SelectItem key="release" textValue="release">Release</SelectItem>
                                        <SelectItem key="beta" textValue="beta">Beta</SelectItem>
                                        <SelectItem key="alpha" textValue="alpha">Alpha</SelectItem>
                                    </Select>
                                </div>

                                {/* Versions Table */}
                                <Table
                                    radius="none"
                                    isHeaderSticky
                                    className="min-h-[400px]"
                                    baseRef={scrollerRef}
                                    bottomContent={
                                        hasMoreVersions ? (
                                            <div className="flex w-full justify-center">
                                                <Spinner ref={loaderRef} color="primary"/>
                                            </div>
                                        ) : null
                                    }
                                    classNames={{
                                        base: "max-h-[520px] overflow-scroll",
                                        table: "min-h-[400px]"
                                    }}
                                >
                                    <TableHeader>
                                        <TableColumn>Name</TableColumn>
                                        <TableColumn>Game Version</TableColumn>
                                        <TableColumn>Platforms</TableColumn>
                                        <TableColumn>Published</TableColumn>
                                        <TableColumn>Downloads</TableColumn>
                                        <TableColumn>Actions</TableColumn>
                                    </TableHeader>
                                    <TableBody
                                        isLoading={versionsLoading}
                                        items={versionsList.items as ModVersion[]}
                                        loadingContent={<Spinner color="primary"/>}
                                    >
                                        {(version:ModVersion) => (
                                            <TableRow key={version.id}>
                                                <TableCell>
                                                    <div className="flex items-center gap-2">
                                                        <Chip
                                                            size="sm"
                                                            color={getVersionTypeColor(version.version_type) as any}
                                                            variant="flat"
                                                            className="min-w-8 text-xs font-bold"
                                                        >
                                                            {getVersionTypeIcon(version.version_type)}
                                                        </Chip>
                                                        <div>
                                                            <div className="font-semibold">{version.version_number}</div>
                                                            <div className="text-xs text-default-500">{version.name}</div>
                                                        </div>
                                                    </div>
                                                </TableCell>
                                                <TableCell>
                                                    <div className="flex flex-wrap gap-1">
                                                        {version.game_versions.slice(0, 3).map(v => (
                                                            <Chip key={v} size="sm" variant="flat">{v}</Chip>
                                                        ))}
                                                        {version.game_versions.length > 3 && (
                                                            <Chip size="sm" variant="flat">+{version.game_versions.length - 3}</Chip>
                                                        )}
                                                    </div>
                                                </TableCell>
                                                <TableCell>
                                                    <div className="flex flex-wrap gap-1">
                                                        {version.loaders.map(loader => (
                                                            <Chip key={loader} size="sm" variant="flat" color="secondary">
                                                                {loader}
                                                            </Chip>
                                                        ))}
                                                    </div>
                                                </TableCell>
                                                <TableCell>{formatDate(version.date_published)}</TableCell>
                                                <TableCell>{formatDownloads(version.downloads)}</TableCell>
                                                <TableCell>
                                                    <Button
                                                        size="sm"
                                                        color="primary"
                                                        radius="none"
                                                        onPress={() => downloadModVersion(version)}
                                                        startContent={<Icon icon="pixelarticons:download"/>}
                                                    >
                                                        Download
                                                    </Button>
                                                </TableCell>
                                            </TableRow>
                                        )}
                                    </TableBody>
                                </Table>
                            </CardBody>
                        </Card>
                    </Tab>
                </Tabs>
            </div>
        </ErrorBoundary>
    );
}