import {ButtonGroup, cn, Divider, Image, Link, Skeleton} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useLocation, useNavigate} from "react-router-dom";
import {Button} from "../../../extended/Button.tsx";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {motion} from "framer-motion";
import {useCallback, useState} from "react";
import type {ModVersion} from "../../../../types/ModTypes.ts";
import {fetchCurseForgeVersions, fetchModrinthVersions} from "../../../../pages/ContentPage.tsx";
import {ModItemContentDrawer} from "./ModItemContentDrawer.tsx";

export type ModItemProps = {
    modId: string;
    platform: "modrinth" | "curseforge";
    description: string;
    iconUrl?: string;
    name: string;
    downloadCount: number;
    author: string;
    categories: string[];
    lastUpdated: Date;
    slug?: string; // Optional slug for CurseForge
}

export function ModItem(props: ModItemProps)
{
    const {server} = useServer();
    const location = useLocation();
    const navigate = useNavigate();
    const [latestVersion, setLatestVersion] = useState<ModVersion | undefined>(undefined);
    const [openDrawer, setOpenDrawer] = useState(false);

    const {
        modId,
        platform,
        description,
        iconUrl,
        name,
        downloadCount,
        author,
        categories,
        lastUpdated,
        slug
    } = props;

    // Build the discover URL with back parameter
    const discoverUrl = `/app/discover/mods/${platform}/${modId}?sid=${server?.id}&back=${encodeURIComponent(location.pathname + location.search)}`;

    const installLatestVersion = useCallback(async () =>
    {


    }, [server, modId, platform, location.pathname, location.search, navigate, discoverUrl]);

    const findLatestVersion = useCallback(async () =>
    {
        let versions: ModVersion[] = [];
        if (!server || !modId || !platform || !server.minecraft_version || !server.server_type) return;
        try
        {

            if (platform === "modrinth")
            {
                versions = await fetchModrinthVersions(modId);
            } else
            {
                versions = await fetchCurseForgeVersions(modId);
            }
            versions = versions.filter(version => version.game_versions.includes(server?.minecraft_version) && version.loaders.includes(server.server_type.toLowerCase()));
            if (versions.length > 0)
            {
                setLatestVersion(versions[0]);
            } else
            {
                setLatestVersion(undefined);
            }
        } catch (error)
        {
            console.error("Failed to fetch mod versions:", error);
        }
    }, [server, modId, platform, location.pathname, location.search, navigate, discoverUrl]);


    return (
        <>
            <ModItemContentDrawer modId={modId} platform={platform} isOpen={openDrawer} onClose={()=>setOpenDrawer(false)} />
            <motion.div
                key={modId}
                className={"flex flex-row gap-2 bg-default-200/50 w-full h-[136px] p-4 font-minecraft-body relative"}
                initial={{opacity: 0, y: 20}}
                animate={{opacity: 1, y: 0}}
                transition={{duration: 0.3}}
                whileHover={{scale: 1.02}}
                whileTap={{scale: 0.99}}
                onClick={() => setOpenDrawer(true)}
                data-platform={platform}
                data-hover-platform={platform}
                data-hover-name={name}
            >
                <div
                    className={"absolute top-0 left-0 w-full h-full z-20 cursor-pointer"}
                />
                <Image src={iconUrl ?? "/favicon.ico"} width={96} height={96} className={"bg-default-100/20 p-2 shrink-0 grow-0 min-w-24 min-h-24"} radius={"none"}/>
                <div className={"flex flex-col gap-2 grow"}>
                    <div className={"flex flex-row gap-2 items-center"}>
                        <p className={"text-2xl font-minecraft-header data-[platform=modrinth]:text-[#1bd96a] data-[platform=curseforge]:text-[#f16436]"} data-platform={platform}>{name}</p>
                        <span className={"text-default-500"}>by {author}</span>
                    </div>
                    <p className={"text-default-700 h-full truncate max-w-[calc(100vw_-_760px)]"}>{description}</p>
                    <div className={"flex flex-row gap-2 text-default-500 truncate max-w-[calc(100vw_-_760px)] h-10"}>
                        {categories.slice(0, 6).map((category, index) => (
                            <>
                                <span key={index} className={"text-sm"}>{category}</span>
                                <Divider orientation={"vertical"}/>
                            </>
                        ))}
                    </div>
                </div>
                <div className={"flex flex-col gap-2 items-end shrink-0"}>
                    <div className={"text-default-700 text-xl flex flex-row items-center gap-1"}><Icon icon={"pixelarticons:download"}/><Downloads count={downloadCount}/> <span className={"text-medium text-default-500"}>downloads</span></div>
                    <div className={"flex flex-row items-center gap-2"}><span className={"text-default-500 flex flex-row gap-1 items-center"}><Icon icon={"pixelarticons:repeat"}/>Updated</span> <LastUpdated date={lastUpdated}/></div>
                    <ButtonGroup className={"gap-2 z-30"}>
                        <Tooltip content={<p>Install <span className={"text-primary"}>{name} {latestVersion?.version_number}</span></p>}>
                            <Button
                                variant={"solid"}
                                onMouseEnter={() => findLatestVersion()}

                                className={
                                    cn(
                                        "data-[platform=curseforge]:bg-[#f16436]",
                                        "data-[platform=modrinth]:bg-[#1bd96a]  data-[platform=modrinth]:!text-background"
                                    )
                                }
                                data-platform={platform}
                            >
                                Install
                            </Button>
                        </Tooltip>
                        <Tooltip content={`Open on ${platform === "modrinth" ? "Modrinth" : "CurseForge"}`}>
                            <Button
                                as={Link}
                                href={platform === "modrinth" ? `https://modrinth.com/mod/${modId}` : `https://www.curseforge.com/minecraft/mc-mods/${slug}`}
                                target={"_blank"}
                                isIconOnly
                                variant={"ghost"}
                                className={
                                    cn(
                                        "data-[platform=curseforge]:border-[#f16436] data-[platform=curseforge]:text-[#f16436] data-[platform=curseforge]:data-[hover]:!bg-[#f16436] data-[platform=curseforge]:data-[hover]:!text-foreground",
                                        "data-[platform=modrinth]:border-[#1bd96a] data-[platform=modrinth]:text-[#1bd96a] data-[platform=modrinth]:data-[hover]:!bg-[#1bd96a] data-[platform=modrinth]:data-[hover]:!text-background"
                                    )
                                }
                                data-platform={platform}
                            >
                                <Icon icon={"pixelarticons:open"}/>
                            </Button>
                        </Tooltip>
                    </ButtonGroup>
                </div>
            </motion.div>
        </>
    );
}

function LastUpdated({date}: { date: Date })
{
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return <span>{days} days ago</span>;
    if (hours > 0) return <span>{hours} hours ago</span>;
    if (minutes > 0) return <span>{minutes} minutes ago</span>;
    return <span className={"text-default-700"}>{seconds} seconds ago</span>;
}


function Downloads({count}: { count: number })
{
    // Format the download as a human-readable string
    // e.g., 1,234,567 -> "1.2M"
    if (count >= 1_000_000) return <span>{(count / 1_000_000).toFixed(1)}M</span>;
    if (count >= 1_000) return <span>{(count / 1_000).toFixed(1)}K</span>;
    return <span>{count}</span>;
}

export function ModItemSkeleton()
{
    return (
        <div className={"flex flex-row gap-2 bg-default-200 w-full h-[200px] p-4 font-minecraft-body"}>
            <Skeleton className={"w-32 h-32 bg-default-100"}/>
            <div className={"flex flex-col gap-2 grow"}>
                <div className={"flex flex-row gap-2 items-center"}>
                    <Skeleton className={"w-48 h-8"}/>
                    <Skeleton className={"w-24 h-6"}/>
                </div>
                <Skeleton className={"w-full h-16"}/>
                <div className={"flex flex-row gap-2"}>
                    <Skeleton className={"w-16 h-4"}/>
                    <Skeleton className={"w-20 h-4"}/>
                    <Skeleton className={"w-18 h-4"}/>
                </div>
            </div>
            <div className={"flex flex-col gap-2 items-end shrink-0"}>
                <Skeleton className={"w-32 h-6"}/>
                <Skeleton className={"w-28 h-5"}/>
                <Skeleton className={"w-36 h-10 mt-auto"}/>
            </div>
        </div>
    );
}