import {Button, Divider, Image, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {InstalledMod, useServer} from "../../../../../providers/ServerProvider.tsx";
import React from "react";
import {Tooltip} from "../../../../extended/Tooltip.tsx";
import {useMessage} from "../../../../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../../../../MessageModal.tsx";

export type InstalledModItemProps = InstalledMod;

export function InstalledModItem(props: InstalledModItemProps): React.JSX.Element
{
    const {removeMod, server} = useServer();
    const {open} = useMessage();
    const {
        mod_id,
        name,
        description,
        version,
        authors,
        icon,
        modrinth_id,
        curseforge_id,
        filename
    } = props;

    // Use the mod icon endpoint if available, otherwise fallback to converting bytes or default icon
    const iconUrl = server?.id
        ? `/api/server/${server.id}/mod/${mod_id}/icon`
        : icon && icon.length > 0
            ? `data:image/png;base64,${btoa(String.fromCharCode(...icon))}`
            : "/favicon.ico";

    const handleRemoveMod = async () =>
    {
        // Show confirmation dialog or any other UI feedback before removing
        let result = await open({
            title: "Remove Mod",
            body: `Are you sure you want to remove the mod "${name}"? This action cannot be undone.`,
            responseType: MessageResponseType.OkayCancel
        });
        if (!result) return; // User cancelled
        try
        {
            await removeMod(mod_id);
            // Optionally show success message or trigger refresh
        } catch (error)
        {
            console.error("Failed to remove mod:", error);
            // Optionally show error message
        }
    };

    return (
        <div className={"flex flex-row gap-2 bg-default-200/50 w-full h-[200px] p-4 font-minecraft-body"}>
            <Image
                src={iconUrl}
                width={128}
                height={128}
                className={"bg-default-100/20 p-2 shrink-0 grow-0 min-w-32 min-h-32"}
                radius={"none"}
                fallbackSrc="/favicon.ico"
            />
            <div className={"flex flex-col gap-2 grow"}>
                <div className={"flex flex-row gap-2 items-center"}>
                    <span className={"text-2xl font-minecraft-header text-foreground"}>{name}</span>
                    <span className={"text-default-500"}>by {authors.join(", ")}</span>
                </div>
                <p className={"text-default-700 h-full"}>{description}</p>
                <div className={"flex flex-row gap-2 text-default-500 items-center"}>
                    <span className={"text-sm"}>Version: {version}</span>
                    <Divider orientation={"vertical"}/>
                    <span className={"text-sm"}>File: {filename}</span>
                </div>
            </div>
            <div className={"flex flex-row gap-2 items-start shrink-0"}>
                {modrinth_id && (
                    <Tooltip content={"Show Modrinth Page"}>
                        <Button
                            as={Link}
                            href={`/app/discover/mods/modrinth/${modrinth_id}?sid=${server?.id}&back=${encodeURIComponent(location.pathname + location.search)}`}
                            isIconOnly
                            radius={"none"}
                            variant={"ghost"}
                            className={
                                "border-[#1bd96a] text-[#1bd96a] data-[hover]:!bg-[#1bd96a] data-[hover]:!text-background"
                            }
                        >
                            <Icon icon={"simple-icons:modrinth"}/>
                        </Button>
                    </Tooltip>
                )}
                {curseforge_id && (
                    <Tooltip content={"Show CurseForge Page"}>
                        <Button
                            as={Link}
                            href={`/app/discover/mods/curseforge/${curseforge_id}?sid=${server?.id}&back=${encodeURIComponent(location.pathname + location.search)}`}
                            target={"_blank"}
                            radius={"none"}
                            variant={"ghost"}
                            className={
                                "border-[#f16436] text-[#f16436] data-[hover]:!bg-[#f16436] data-[hover]:!text-foreground"
                            }
                            isIconOnly
                        >
                            <Icon icon={"simple-icons:curseforge"}/>
                        </Button>
                    </Tooltip>
                )}
                <Tooltip content={"Delete Mod"}>
                    <Button
                        onPress={handleRemoveMod}
                        radius={"none"}
                        variant={"ghost"}
                        className={"border-danger text-danger data-[hover]:!bg-danger data-[hover]:!text-white"}
                        isIconOnly
                    >
                        <Icon icon={"pixelarticons:trash"}/>
                    </Button>
                </Tooltip>
            </div>
        </div>
    );
}
