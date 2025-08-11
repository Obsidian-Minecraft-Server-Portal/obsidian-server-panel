import {useEffect, useState} from "react";
import {Chip, Input, SelectItem, Spinner, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useInfiniteScroll} from "@heroui/use-infinite-scroll";
import {useAsyncList} from "@react-stately/data";
import {ModVersion, ServerInfo} from "../../types/ModTypes";
import {Select} from "../extended/Select.tsx";
import {Button} from "../extended/Button.tsx";

interface ModVersionsProps
{
    modVersions: ModVersion[];
    versionsLoading: boolean;
    server?: ServerInfo;
    serverId?: string;
    onDownloadVersion: (version: ModVersion) => Promise<void>;
}

export function ModVersions({modVersions, versionsLoading, server, serverId, onDownloadVersion}: ModVersionsProps)
{
    const [versionFilter, setVersionFilter] = useState("");
    const [gameVersionFilter, setGameVersionFilter] = useState("");
    const [loaderFilter, setLoaderFilter] = useState("");
    const [typeFilter, setTypeFilter] = useState("");
    const [hasMoreVersions, setHasMoreVersions] = useState(false);
    const [isInstallingVersion, setIsInstallingVersion] = useState(false);
    const [installedVersion, setInstalledVersion] = useState<ModVersion|undefined>(undefined);

    const versionsPerPage = 50;

    // Set default filters based on server configuration
    useEffect(() =>
    {
        if (server && serverId && modVersions.length > 0)
        {
            // Set game version filter to match server version
            if (server.minecraft_version && !gameVersionFilter)
            {
                const serverVersion = server.minecraft_version;
                const availableVersions = Array.from(new Set(modVersions.flatMap(v => v.game_versions)));
                if (availableVersions.includes(serverVersion))
                {
                    setGameVersionFilter(serverVersion);
                }
            }

            // Set loader filter to match server type
            if (server.server_type && !loaderFilter)
            {
                const serverLoader = server.server_type.toLowerCase();
                const availableLoaders = Array.from(new Set(modVersions.flatMap(v => v.loaders.map(l => l.toLowerCase()))));
                if (availableLoaders.includes(serverLoader))
                {
                    setLoaderFilter(serverLoader);
                }
            }
        }
    }, [server, serverId, modVersions, gameVersionFilter, loaderFilter]);

    const versionsList = useAsyncList({
        // @ts-ignore
        async load({cursor})
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

    return (
        <div className="p-6 h-[calc(100dvh_-_300px)]">
            {/* Filters */}
            <div className="flex gap-4 mb-6 flex-nowrap">
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
                removeWrapper
                className="min-h-[400px] h-full"
                baseRef={scrollerRef}
                bottomContent={
                    hasMoreVersions ? (
                        <div className="flex w-full justify-center">
                            <Spinner ref={loaderRef} color="primary"/>
                        </div>
                    ) : null
                }
                classNames={{
                    base: "overflow-scroll",
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
                    {(version: ModVersion) => (
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
                                        <Chip key={loader} size="sm" variant="flat">
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
                                    onPress={async () =>
                                    {
                                        setIsInstallingVersion(true)
                                        await onDownloadVersion(version);
                                        setInstalledVersion(version)
                                        setIsInstallingVersion(false)
                                    }}
                                    startContent={<Icon icon="pixelarticons:download"/>}
                                    variant={"ghost"}
                                >
                                    Download
                                </Button>
                            </TableCell>
                        </TableRow>
                    )}
                </TableBody>
            </Table>
        </div>
    );
}
