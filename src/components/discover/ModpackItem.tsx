import {ButtonGroup, Divider, Image, Link, Skeleton} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Button} from "../extended/Button.tsx";
import {Tooltip} from "../extended/Tooltip.tsx";
import {motion} from "framer-motion";
import {useState} from "react";
import type {ModpackItemProps} from "../../types/ModpackTypes.ts";
import {ModpackItemContentDrawer} from "./ModpackItemContentDrawer.tsx";

export function ModpackItem(props: ModpackItemProps)
{
    const [openDrawer, setOpenDrawer] = useState(false);

    const {
        packId,
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

    const getPlatformColor = () =>
    {
        switch (platform)
        {
            case "modrinth":
                return "#1bd96a";
            case "curseforge":
                return "#f16436";
            case "atlauncher":
                return "#3498db";
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
        switch (platform)
        {
            case "modrinth":
                return `https://modrinth.com/modpack/${packId}`;
            case "curseforge":
                return `https://www.curseforge.com/minecraft/modpacks/${slug || packId}`;
            case "atlauncher":
                return `https://atlauncher.com/pack/${slug || packId}`;
            case "technic":
                return `https://www.technicpack.net/modpack/${slug || packId}`;
            default:
                return "#";
        }
    };

    return (
        <>
            <ModpackItemContentDrawer
                packId={packId}
                platform={platform}
                isOpen={openDrawer}
                onClose={() => setOpenDrawer(false)}
            />
            <motion.div
                key={packId}
                className={"flex flex-row gap-2 bg-default-200/50 w-full h-[136px] p-4 font-minecraft-body relative"}
                initial={{opacity: 0, y: 20}}
                animate={{opacity: 1, y: 0}}
                transition={{duration: 0.3}}
                whileHover={{scale: 1.02}}
                whileTap={{scale: 0.99}}
                onClick={() => setOpenDrawer(true)}
                data-platform={platform}
            >
                <div
                    className={"absolute top-0 left-0 w-full h-full z-20 cursor-pointer"}
                />
                <Image
                    src={iconUrl ?? "/favicon.ico"}
                    width={96}
                    height={96}
                    className={"bg-default-100/20 p-2 shrink-0 grow-0 min-w-24 min-h-24"}
                    radius={"none"}
                />
                <div className={"flex flex-col gap-2 grow"}>
                    <div className={"flex flex-row gap-2 items-center"}>
                        <p
                            className={"text-2xl font-minecraft-header"}
                            style={{color: getPlatformColor()}}
                        >
                            {name}
                        </p>
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
                    {downloadCount > 0 && (
                        <div className={"text-default-700 text-xl flex flex-row items-center gap-1"}>
                            <Icon icon={"pixelarticons:download"}/>
                            <Downloads count={downloadCount}/>
                            <span className={"text-medium text-default-500"}>downloads</span>
                        </div>
                    )}
                    <div className={"flex flex-row items-center gap-2"}>
                        <span className={"text-default-500 flex flex-row gap-1 items-center"}>
                            <Icon icon={"pixelarticons:repeat"}/>Updated
                        </span>
                        <LastUpdated date={lastUpdated}/>
                    </div>
                    <ButtonGroup className={"gap-2 z-30"}>
                        <Tooltip content={`View ${name} details`}>
                            <Button
                                variant={"solid"}
                                style={{backgroundColor: getPlatformColor()}}
                                className={platform === "modrinth" ? "!text-background" : ""}
                            >
                                View Details
                            </Button>
                        </Tooltip>
                        <Tooltip content={`Open on ${getPlatformName()}`}>
                            <Button
                                as={Link}
                                href={getExternalUrl()}
                                target={"_blank"}
                                isIconOnly
                                variant={"ghost"}
                                style={{
                                    borderColor: getPlatformColor(),
                                    color: getPlatformColor()
                                }}
                                className={"data-[hover]:!bg-opacity-100"}
                                onMouseEnter={(e: any) =>
                                {
                                    e.currentTarget.style.backgroundColor = getPlatformColor();
                                    e.currentTarget.style.color = platform === "modrinth" ? "#000" : "#fff";
                                }}
                                onMouseLeave={(e: any) =>
                                {
                                    e.currentTarget.style.backgroundColor = "transparent";
                                    e.currentTarget.style.color = getPlatformColor();
                                }}
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
    if (count >= 1_000_000) return <span>{(count / 1_000_000).toFixed(1)}M</span>;
    if (count >= 1_000) return <span>{(count / 1_000).toFixed(1)}K</span>;
    return <span>{count}</span>;
}

export function ModpackItemSkeleton()
{
    return (
        <div className={"flex flex-row gap-2 bg-default-200 w-full h-[136px] p-4 font-minecraft-body"}>
            <Skeleton className={"w-24 h-24 bg-default-100"}/>
            <div className={"flex flex-col gap-2 grow"}>
                <div className={"flex flex-row gap-2 items-center"}>
                    <Skeleton className={"w-48 h-8"}/>
                    <Skeleton className={"w-24 h-6"}/>
                </div>
                <Skeleton className={"w-full h-12"}/>
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
