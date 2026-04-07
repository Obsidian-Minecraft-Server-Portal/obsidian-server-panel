import {useEffect, useState} from "react";
import {Chip, ListBoxItem, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {Input} from "../extended/Input.tsx";
import {Icon} from "@iconify-icon/react";
// @ts-ignore - module may not exist in v3
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

export function ModVersions({modVersions, server, serverId, onDownloadVersion}: ModVersionsProps)
{
    const [versionFilter, setVersionFilter] = useState("");
    const [gameVersionFilter, setGameVersionFilter] = useState("");
    const [loaderFilter, setLoaderFilter] = useState("");
    const [typeFilter, setTypeFilter] = useState("");
    const [_hasMoreVersions, setHasMoreVersions] = useState(false);
    const [_isInstallingVersion, setIsInstallingVersion] = useState(false);
    const [_installedVersion, setInstalledVersion] = useState<ModVersion|undefined>(undefined);

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
                    className="max-w-xs rounded-none"
                    startContent={<Icon icon="pixelarticons:search"/>}
                />
                <Select
                    placeholder="Game Version"
                    selectedKey={gameVersionFilter || undefined}
                    onSelectionChange={(keys: any) => setGameVersionFilter([...keys][0] as string || "")}
                    className="max-w-xs"
                >
                    {Array.from(new Set(modVersions.flatMap(v => v.game_versions))).map(version => (
                        <ListBoxItem key={version} textValue={version}>{version}</ListBoxItem>
                    ))}
                </Select>
                <Select
                    placeholder="Loader"
                    selectedKey={loaderFilter || undefined}
                    onSelectionChange={(keys: any) => setLoaderFilter([...keys][0] as string || "")}
                    className="max-w-xs"
                >
                    {Array.from(new Set(modVersions.flatMap(v => v.loaders))).map(loader => (
                        <ListBoxItem key={loader} textValue={loader}>{loader}</ListBoxItem>
                    ))}
                </Select>
                <Select
                    placeholder="Release Type"
                    selectedKey={typeFilter || undefined}
                    onSelectionChange={(keys: any) => setTypeFilter([...keys][0] as string || "")}
                    className="max-w-xs"
                >
                    <ListBoxItem key="release" textValue="release">Release</ListBoxItem>
                    <ListBoxItem key="beta" textValue="beta">Beta</ListBoxItem>
                    <ListBoxItem key="alpha" textValue="alpha">Alpha</ListBoxItem>
                </Select>
            </div>

            {/* Versions Table */}
            <Table
                className="min-h-[400px] h-full rounded-none overflow-scroll"
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
                    items={versionsList.items as ModVersion[]}
                >
                    {(version: ModVersion) => (
                        <TableRow key={version.id}>
                            <TableCell>
                                <div className="flex items-center gap-2">
                                    <Chip
                                        color={getVersionTypeColor(version.version_type) as any}
                                        variant="soft"
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
                                        <Chip key={v} size="sm" variant="soft">{v}</Chip>
                                    ))}
                                    {version.game_versions.length > 3 && (
                                        <Chip size="sm" variant="soft">+{version.game_versions.length - 3}</Chip>
                                    )}
                                </div>
                            </TableCell>
                            <TableCell>
                                <div className="flex flex-wrap gap-1">
                                    {version.loaders.map(loader => (
                                        <Chip key={loader} size="sm" variant="soft">
                                            {loader}
                                        </Chip>
                                    ))}
                                </div>
                            </TableCell>
                            <TableCell>{formatDate(version.date_published)}</TableCell>
                            <TableCell>{formatDownloads(version.downloads)}</TableCell>
                            <TableCell>
                                <Button
                                    variant={"outline"}
                                    onPress={async () =>
                                    {
                                        setIsInstallingVersion(true)
                                        await onDownloadVersion(version);
                                        setInstalledVersion(version)
                                        setIsInstallingVersion(false)
                                    }}
                                >
                                    <Icon icon="pixelarticons:download"/> Download
                                </Button>
                            </TableCell>
                        </TableRow>
                    )}
                </TableBody>
            </Table>
        </div>
    );
}
