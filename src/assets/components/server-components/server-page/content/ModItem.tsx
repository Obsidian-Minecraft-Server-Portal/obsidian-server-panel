import {Button, cn, Divider, Image, Link, Skeleton} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useLocation} from "react-router-dom";

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

    return (
        <div key={modId} className={"flex flex-row gap-2 bg-default-200/50 w-full h-[136px] p-4 font-minecraft-body"}>
            <Image src={iconUrl ?? "/favicon.ico"} width={96} height={96} className={"bg-default-100/20 p-2 shrink-0 grow-0 min-w-24 min-h-24"} radius={"none"}/>
            <div className={"flex flex-col gap-2 grow"}>
                <div className={"flex flex-row gap-2 items-center"}>
                    <Link className={"text-2xl font-minecraft-header data-[platform=curseforge]:text-[#f16436]"} href={discoverUrl} data-platform={platform}>{name}</Link>
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
                <Button
                    as={Link}
                    href={platform === "modrinth" ? `https://modrinth.com/mod/${modId}` : `https://www.curseforge.com/minecraft/mc-mods/${slug}`}
                    target={"_blank"}
                    showAnchorIcon
                    radius={"none"}
                    variant={"ghost"}
                    className={
                        cn(
                            "mt-auto",
                            "data-[platform=curseforge]:border-[#f16436] data-[platform=curseforge]:text-[#f16436] data-[platform=curseforge]:data-[hover]:!bg-[#f16436] data-[platform=curseforge]:data-[hover]:!text-foreground",
                            "data-[platform=modrinth]:border-[#1bd96a] data-[platform=modrinth]:text-[#1bd96a] data-[platform=modrinth]:data-[hover]:!bg-[#1bd96a] data-[platform=modrinth]:data-[hover]:!text-background"
                        )
                    }
                    data-platform={platform}
                >
                    Open on {platform === "modrinth" ? "Modrinth" : "CurseForge"}
                </Button>
            </div>
        </div>
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