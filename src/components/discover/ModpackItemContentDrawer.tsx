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
import {useEffect, useMemo, useState} from "react";
import {Icon} from "@iconify-icon/react";
import type {ModpackDetails, ModpackVersion, ModpackPlatform} from "../../types/ModpackTypes.ts";
import {
    fetchModrinthModpackDetails,
    fetchModrinthModpackVersions,
    fetchCurseForgeModpackDetails,
    fetchCurseForgeModpackVersions,
    fetchATLauncherModpackDetails,
    fetchATLauncherModpackVersions,
    fetchTechnicModpackDetails,
    fetchTechnicModpackVersions
} from "../../utils/modpack-api.ts";
import {ModDescription} from "./ModDescription.tsx";
import {ModChangelog} from "./ModChangelog.tsx";

type ModpackItemContentDrawerProps = {
    packId: string;
    platform: ModpackPlatform;
} & Omit<DrawerProps, "children">;

export function ModpackItemContentDrawer(props: ModpackItemContentDrawerProps)
{
    const {packId, platform, ...rest} = props;

    const [modpackDetails, setModpackDetails] = useState<ModpackDetails | null>(null);
    const [modpackVersions, setModpackVersions] = useState<ModpackVersion[]>([]);
    const [loading, setLoading] = useState(false);
    const [versionsLoading, setVersionsLoading] = useState(false);
    const [selectedTab, setSelectedTab] = useState("description");

    const formatDate = (dateString: string) => new Date(dateString).toLocaleDateString();
    const formatDownloads = (count: number) =>
    {
        if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
        if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
        return count.toString();
    };

    const getPlatformColor = () =>
    {
        switch (platform)
        {
            case "modrinth":
                return "#1bd96a";
            case "curseforge":
                return "#f16436";
            case "atlauncher":
                return "#89c236";
            case "technic":
                return "#e74c3c";
            default:
                return "#888";
        }
    };

    const getPlatformName = () =>
    {
        switch (platform)
        {
            case "modrinth":
                return "Modrinth";
            case "curseforge":
                return "CurseForge";
            case "atlauncher":
                return "ATLauncher";
            case "technic":
                return "Technic";
            default:
                return platform;
        }
    };

    const getExternalUrl = () =>
    {
        if (!modpackDetails) return "#";
        switch (platform)
        {
            case "modrinth":
                return `https://modrinth.com/modpack/${modpackDetails.slug ?? packId}`;
            case "curseforge":
                return `https://www.curseforge.com/minecraft/modpacks/${modpackDetails.slug ?? packId}`;
            case "atlauncher":
                return `https://atlauncher.com/pack/${modpackDetails.slug ?? packId}`;
            case "technic":
                return `https://www.technicpack.net/modpack/${modpackDetails.slug ?? packId}`;
            default:
                return "#";
        }
    };

    // Fetch modpack details
    useEffect(() =>
    {
        let aborted = false;

        async function loadDetails()
        {
            if (!rest.isOpen || !packId) return;
            try
            {
                setLoading(true);
                let details: ModpackDetails;

                switch (platform)
                {
                    case "modrinth":
                        details = await fetchModrinthModpackDetails(packId);
                        break;
                    case "curseforge":
                        details = await fetchCurseForgeModpackDetails(packId);
                        break;
                    case "atlauncher":
                        details = await fetchATLauncherModpackDetails(packId);
                        break;
                    case "technic":
                        details = await fetchTechnicModpackDetails(packId);
                        break;
                    default:
                        throw new Error(`Unsupported platform: ${platform}`);
                }

                if (aborted) return;
                setModpackDetails(details);
            } catch (e)
            {
                console.error("Failed to fetch modpack details:", e);
                setModpackDetails(null);
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
    }, [rest.isOpen, packId, platform]);

    // Fetch modpack versions
    useEffect(() =>
    {
        let aborted = false;

        async function loadVersions()
        {
            if (!rest.isOpen || !packId) return;
            try
            {
                setVersionsLoading(true);
                let versions: ModpackVersion[];

                switch (platform)
                {
                    case "modrinth":
                        versions = await fetchModrinthModpackVersions(packId);
                        break;
                    case "curseforge":
                        versions = await fetchCurseForgeModpackVersions(packId);
                        break;
                    case "atlauncher":
                        versions = await fetchATLauncherModpackVersions(packId);
                        break;
                    case "technic":
                        versions = await fetchTechnicModpackVersions(packId);
                        break;
                    default:
                        throw new Error(`Unsupported platform: ${platform}`);
                }

                if (aborted) return;
                setModpackVersions(versions);
            } catch (e)
            {
                console.error("Failed to fetch modpack versions:", e);
                setModpackVersions([]);
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
    }, [rest.isOpen, packId, platform]);

    // Reset tab when drawer opens
    useEffect(() =>
    {
        if (rest.isOpen)
        {
            setSelectedTab("description");
        }
    }, [rest.isOpen]);

    const changelog = useMemo(() =>
    {
        return modpackVersions
            .filter(v => v.changelog && v.changelog.trim())
            .map(v => ({
                version: v.version_number,
                version_type: v.version_type,
                date: v.date_published,
                changes: v.changelog || ""
            }))
            .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
    }, [modpackVersions]);

    return (
        <Drawer
            placement={"bottom"}
            size={"full"}
            radius={"none"}
            scrollBehavior={"inside"}
            backdrop={"blur"}
            hideCloseButton={true}
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
                                    src={modpackDetails?.icon_url || "/favicon.ico"}
                                    alt={modpackDetails?.name ?? "Modpack"}
                                    width={40}
                                    height={40}
                                    radius="sm"
                                    className="rounded-md"
                                />
                                <span className="text-xl font-bold">{modpackDetails?.name ?? "Modpack"}</span>
                                <Chip
                                    style={{backgroundColor: getPlatformColor()}}
                                    className={platform === "modrinth" ? "text-black" : "text-white"}
                                    variant="flat"
                                    size="sm"
                                >
                                    {getPlatformName()}
                                </Chip>
                                <div className="ml-auto flex items-center gap-2">
                                    {modpackDetails && (
                                        <Button
                                            as={Link}
                                            href={getExternalUrl()}
                                            target="_blank"
                                            radius="none"
                                            variant="solid"
                                            style={{backgroundColor: getPlatformColor()}}
                                            className={platform === "modrinth" ? "text-black" : "text-white"}
                                            endContent={<Icon icon="pixelarticons:external-link"/>}
                                        >
                                            Open on {getPlatformName()}
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
                        ) : !modpackDetails ? (
                            <div className="text-danger">Failed to load modpack information.</div>
                        ) : (
                            <div className="w-full">
                                <div className="flex flex-col gap-2 mb-4">
                                    <p className="text-default-600">{modpackDetails.description}</p>
                                    <div className="flex flex-wrap gap-4 text-sm">
                                        {modpackDetails.downloads > 0 && (
                                            <div className="flex items-center gap-1">
                                                <Icon icon="pixelarticons:download"/>
                                                <span className="font-semibold">{formatDownloads(modpackDetails.downloads)}</span>
                                                <span className="text-default-500">downloads</span>
                                            </div>
                                        )}
                                        {modpackDetails.followers && (
                                            <div className="flex items-center gap-1">
                                                <Icon icon="pixelarticons:heart"/>
                                                <span className="font-semibold">{formatDownloads(modpackDetails.followers)}</span>
                                                <span className="text-default-500">followers</span>
                                            </div>
                                        )}
                                        {modpackDetails.updated && (
                                            <div className="flex items-center gap-1">
                                                <Icon icon="pixelarticons:calendar"/>
                                                <span className="text-default-500">Updated {formatDate(modpackDetails.updated)}</span>
                                            </div>
                                        )}
                                    </div>
                                    <div className="flex flex-wrap gap-2">
                                        {modpackDetails.categories.slice(0, 6).map(category => (
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
                                        <ModDescription modDetails={modpackDetails as any}/>
                                    </Tab>
                                    <Tab key="changelog" title="Changelog">
                                        <ModChangelog
                                            changelog={changelog}
                                            changelogPage={1}
                                            onLoadMore={() => {}}
                                        />
                                    </Tab>
                                    <Tab key="versions" title="Versions">
                                        <div className="p-4">
                                            {versionsLoading ? (
                                                <div className="space-y-2">
                                                    <Skeleton className="h-12 w-full"/>
                                                    <Skeleton className="h-12 w-full"/>
                                                    <Skeleton className="h-12 w-full"/>
                                                </div>
                                            ) : modpackVersions.length === 0 ? (
                                                <div className="text-default-500">No versions available</div>
                                            ) : (
                                                <div className="space-y-2">
                                                    {modpackVersions.map(version => (
                                                        <div
                                                            key={version.id}
                                                            className="flex items-center justify-between p-3 bg-default-100 rounded-md"
                                                        >
                                                            <div>
                                                                <div className="font-semibold">{version.version_number}</div>
                                                                <div className="text-sm text-default-500">
                                                                    {version.game_versions.join(", ")}
                                                                </div>
                                                            </div>
                                                            <Chip
                                                                size="sm"
                                                                color={
                                                                    version.version_type === "release"
                                                                        ? "success"
                                                                        : version.version_type === "beta"
                                                                            ? "warning"
                                                                            : "danger"
                                                                }
                                                            >
                                                                {version.version_type}
                                                            </Chip>
                                                        </div>
                                                    ))}
                                                </div>
                                            )}
                                        </div>
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
