import {
    Drawer,
    DrawerContent,
    DrawerFooter,
    DrawerProps,
    Tabs,
    Tab,
    Image,
    Chip,
    Button,
    Link,
    Skeleton
} from "@heroui/react";
import {useCallback, useEffect, useState} from "react";
import {Icon} from "@iconify-icon/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useMessage} from "../../../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../../../MessageModal.tsx";
import type {ChangelogEntry, ModDetails, ModVersion} from "../../../../types/ModTypes.ts";
import {fetchCurseForgeVersions, fetchModrinthVersions} from "../../../../pages/ContentPage.tsx";
import {ModDescription} from "../../../discover/ModDescription.tsx";
import {ModChangelog} from "../../../discover/ModChangelog.tsx";
import {ModVersions} from "../../../discover/ModVersions.tsx";

type ModItemContentDrawerProps = {
    modId: string;
    platform: "modrinth" | "curseforge";
} & Omit<DrawerProps, "children">;

export function ModItemContentDrawer(props: ModItemContentDrawerProps)
{
    const {server, installMod} = useServer();
    const {open} = useMessage();
    const {modId, platform, ...rest} = props;

    const [modDetails, setModDetails] = useState<ModDetails | null>(null);
    const [modVersions, setModVersions] = useState<ModVersion[]>([]);
    const [changelog, setChangelog] = useState<ChangelogEntry[]>([]);
    const [loading, setLoading] = useState(false);
    const [versionsLoading, setVersionsLoading] = useState(false);
    const [selectedTab, setSelectedTab] = useState("description");
    const [changelogPage, setChangelogPage] = useState(1);

    const formatDate = (dateString: string) => new Date(dateString).toLocaleDateString();
    const formatDownloads = (count: number) =>
    {
        if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
        if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
        return count.toString();
    };

    // Details fetchers (scoped here for the Drawer)
    const fetchModrinthProject = useCallback(async (projectId: string): Promise<ModDetails> =>
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
    }, []);

    const fetchCurseForgeProject = useCallback(async (projectId: string): Promise<ModDetails> =>
    {
        const response = await fetch(`/api/platform/curseforge/mod/${projectId}`);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
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
    }, []);

    const downloadModVersion = useCallback(async (version: ModVersion) =>
    {
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

        const serverVersion = server.minecraft_version;
        const serverLoader = server.server_type;

        let compatible = true;
        const warnings: string[] = [];

        if (serverVersion && !version.game_versions.includes(serverVersion))
        {
            compatible = false;
            warnings.push(`This mod version supports Minecraft ${version.game_versions.join(", ")} but your server runs ${serverVersion}`);
        }

        if (
            serverLoader &&
            !serverLoader.equalsIgnoreCase?.("vanilla") &&
            !version.loaders.map(i => i.toLowerCase()).includes(serverLoader.toLowerCase())
        )
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
    }, [server, installMod, open, modDetails, platform]);

    // Load content when opening
    useEffect(() =>
    {
        let aborted = false;

        async function loadDetails()
        {
            if (!rest.isOpen || !modId) return;
            try
            {
                setLoading(true);
                const details =
                    platform === "modrinth"
                        ? await fetchModrinthProject(modId)
                        : await fetchCurseForgeProject(modId);
                if (aborted) return;
                setModDetails(details);
            } catch (e)
            {
                console.error("Failed to fetch project details:", e);
                setModDetails(null);
            } finally
            {
                if (!aborted) setLoading(false);
            }
        }

        loadDetails();
        return () =>
        {
            aborted = true;
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [rest.isOpen]);

    // Load versions and build changelog
    useEffect(() =>
    {
        let aborted = false;

        async function loadVersions()
        {
            if (!rest.isOpen || !modId) return;
            try
            {
                setVersionsLoading(true);
                const versions =
                    platform === "modrinth"
                        ? await fetchModrinthVersions(modId)
                        : await fetchCurseForgeVersions(modId);
                if (aborted) return;

                setModVersions(versions);

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
            } catch (e)
            {
                console.error("Failed to fetch versions:", e);
                setModVersions([]);
                setChangelog([]);
            } finally
            {
                if (!aborted) setVersionsLoading(false);
            }
        }

        loadVersions();
        return () =>
        {
            aborted = true;
        };
    }, [rest.isOpen]);

    // Reset tab when mod changes or drawer opens
    useEffect(() =>
    {
        if (rest.isOpen)
        {
            setSelectedTab("description");
            setChangelogPage(1);
        }
    }, [rest.isOpen]);

    return (
        <Drawer
            placement={"bottom"}
            size={"full"}
            radius={"none"}
            scrollBehavior={"inside"}
            backdrop={"blur"}
            hideCloseButton={true}
            onMouseDown={(e) =>
            {
                e.stopPropagation();
                console.log("Mouse Down", e);
            }}
            {...rest}
        >
            <DrawerContent className="p-4 font-minecraft-body">
                {onClose => (
                    <>
                        {loading ? (
                            <div className="flex items-center gap-3 w-full">
                                <Skeleton className="w-10 h-10 rounded-md"/>
                                <Skeleton className="h-6 w-48"/>
                                <Skeleton className="h-6 w-20 ml-auto"/>
                            </div>
                        ) : (
                            <div className="flex items-center gap-3 w-full">
                                <Image
                                    src={modDetails?.icon_url || "/favicon.ico"}
                                    alt={modDetails?.name ?? "Mod"}
                                    width={40}
                                    height={40}
                                    radius="sm"
                                    className="rounded-md"
                                />
                                <span className="text-xl font-bold">{modDetails?.name ?? "Mod"}</span>
                                <Chip
                                    color={platform === "modrinth" ? "success" : "warning"}
                                    variant="flat"
                                    size="sm"
                                >
                                    {platform === "modrinth" ? "Modrinth" : "CurseForge"}
                                </Chip>
                                <div className="ml-auto flex items-center gap-2">
                                    {modDetails && (
                                        <Button
                                            as={Link}
                                            href={
                                                platform === "modrinth"
                                                    ? `https://modrinth.com/mod/${modDetails.slug ?? modId}`
                                                    : `https://www.curseforge.com/minecraft/mc-mods/${modDetails.slug ?? modId}`
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
                                    )}

                                    <Button
                                        isIconOnly
                                        radius="none"
                                        variant="solid"
                                        onPress={onClose}
                                    >
                                        <Icon icon="pixelarticons:close"/>
                                    </Button>
                                </div>
                            </div>
                        )}

                        {loading ? (
                            <div className="flex gap-4">
                                <Skeleton className="w-24 h-24 rounded-lg"/>
                                <div className="flex-1 space-y-3">
                                    <Skeleton className="w-2/3 h-6"/>
                                    <Skeleton className="w-full h-16"/>
                                    <div className="flex gap-3">
                                        <Skeleton className="w-20 h-5"/>
                                        <Skeleton className="w-20 h-5"/>
                                        <Skeleton className="w-20 h-5"/>
                                    </div>
                                </div>
                            </div>
                        ) : !modDetails ? (
                            <div className="text-danger">Failed to load mod information.</div>
                        ) : (
                            <div className="w-full">
                                <div className="flex flex-col gap-2 mb-4">
                                    <p className="text-default-600">{modDetails.description}</p>
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
                        )}
                    </>
                )}
            </DrawerContent>

            <DrawerFooter className="font-minecraft-body">
                <div className="ml-auto">
                    <Button onPress={rest.onClose} radius="none" variant="flat">
                        Close
                    </Button>
                </div>
            </DrawerFooter>
        </Drawer>
    );
}
