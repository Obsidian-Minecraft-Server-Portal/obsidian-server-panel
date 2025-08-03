import {Button, Divider, Image, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {InstalledMod, useServer} from "../../../../../providers/ServerProvider.tsx";
import React, {useState, useEffect} from "react";
import {Tooltip} from "../../../../extended/Tooltip.tsx";
import {useMessage} from "../../../../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../../../../MessageModal.tsx";
import {checkCurseForgeUpdate, checkModrinthUpdate, ModUpdateInfo} from "../../../../../ts/mod-updates.ts";

export type InstalledModItemProps = InstalledMod;

export function InstalledModItem(props: InstalledModItemProps): React.JSX.Element
{
    const {removeMod, server, loadServer, installMod} = useServer();
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

    const [modrinthUpdateInfo, setModrinthUpdateInfo] = useState<ModUpdateInfo | null>(null);
    const [curseforgeUpdateInfo, setCurseforgeUpdateInfo] = useState<ModUpdateInfo | null>(null);
    const [isCheckingUpdates, setIsCheckingUpdates] = useState(false);
    const [isUpdating, setIsUpdating] = useState<"modrinth" | "curseforge" | null>(null);

    // Check for updates when component mounts or server changes
    useEffect(() =>
    {
        if (!server?.minecraft_version || !server?.server_type) return;

        const checkForUpdates = async () =>
        {
            setIsCheckingUpdates(true);
            try
            {
                // Check Modrinth updates
                if (modrinth_id)
                {
                    const modrinthUpdate = await checkModrinthUpdate(
                        modrinth_id,
                        version,
                        server.minecraft_version,
                        server.server_type
                    );
                    setModrinthUpdateInfo(modrinthUpdate);
                }

                // Check CurseForge updates
                if (curseforge_id)
                {
                    const curseforgeUpdate = await checkCurseForgeUpdate(
                        curseforge_id,
                        filename, // Use filename for CurseForge comparison
                        server.minecraft_version,
                        server.server_type
                    );
                    setCurseforgeUpdateInfo(curseforgeUpdate);
                }
            } catch (error)
            {
                console.error("Error checking for mod updates:", error);
            } finally
            {
                setIsCheckingUpdates(false);
            }
        };

        checkForUpdates();
    }, [modrinth_id, curseforge_id, version, filename, server?.minecraft_version, server?.server_type]);

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
            responseType: MessageResponseType.OkayCancel,
            severity: "danger"
        });
        if (!result) return; // User cancelled
        try
        {
            await removeMod(mod_id);
            if (server?.id) await loadServer(server?.id);
        } catch (error)
        {
            console.error("Failed to remove mod:", error);
            // Optionally show error message
        }
    };

    const handleUpdateMod = async (source: "modrinth" | "curseforge") =>
    {
        const updateInfo = source === "modrinth" ? modrinthUpdateInfo : curseforgeUpdateInfo;
        if (!updateInfo || !server?.id) return;

        const result = await open({
            title: "Update Mod",
            body: `Update "${name}" from version ${version} to ${updateInfo.version}?`,
            responseType: MessageResponseType.OkayCancel,
            severity: "info"
        });

        if (!result) return;

        setIsUpdating(source);
        try
        {
            // Install the new version
            await installMod({
                downloadUrl: updateInfo.downloadUrl,
                filename: updateInfo.filename
            });

            // Remove the old version
            await removeMod(mod_id);

            // Reload server to refresh mod list
            await loadServer(server.id);

            // Clear update info since we've updated
            if (source === "modrinth")
            {
                setModrinthUpdateInfo(null);
            } else
            {
                setCurseforgeUpdateInfo(null);
            }

            await open({
                title: "Update Complete",
                body: `Successfully updated "${name}" to version ${updateInfo.version}`,
                responseType: MessageResponseType.Close,
                severity: "success"
            });
        } catch (error)
        {
            console.error("Failed to update mod:", error);
            await open({
                title: "Update Failed",
                body: `Failed to update "${name}": ${error instanceof Error ? error.message : "Unknown error"}`,
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        } finally
        {
            setIsUpdating(null);
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
                    <div className={"flex flex-row gap-2 items-start shrink-0"}>
                        {modrinthUpdateInfo && (
                            <Tooltip content={`Update to version ${modrinthUpdateInfo.version} on Modrinth`}>
                                <Button
                                    onPress={() => handleUpdateMod("modrinth")}
                                    radius={"none"}
                                    variant={"ghost"}
                                    isIconOnly
                                    isLoading={isUpdating === "modrinth"}
                                    isDisabled={isUpdating !== null}
                                    className={
                                        "border-[#1bd96a] text-[#1bd96a] data-[hover]:!bg-[#1bd96a] data-[hover]:!text-background"
                                    }
                                >
                                    <Icon icon={"pixelarticons:download"}/>
                                </Button>
                            </Tooltip>
                        )}
                        {curseforgeUpdateInfo && (
                            <Tooltip content={`Update to version ${curseforgeUpdateInfo.version} on CurseForge`}>
                                <Button
                                    onPress={() => handleUpdateMod("curseforge")}
                                    radius={"none"}
                                    variant={"ghost"}
                                    isIconOnly
                                    isLoading={isUpdating === "curseforge"}
                                    isDisabled={isUpdating !== null}
                                    className={
                                        "border-[#f16436] text-[#f16436] data-[hover]:!bg-[#f16436] data-[hover]:!text-foreground"
                                    }
                                >
                                    <Icon icon={"pixelarticons:download"}/>
                                </Button>
                            </Tooltip>
                        )}
                        {isCheckingUpdates && (
                            <Tooltip content="Checking for updates...">
                                <Button
                                    radius={"none"}
                                    variant={"ghost"}
                                    isIconOnly
                                    isLoading={true}
                                    className={"border-default-300 text-default-300"}
                                >
                                    <Icon icon={"pixelarticons:reload"}/>
                                </Button>
                            </Tooltip>
                        )}
                    </div>
                    <span className={"text-2xl font-minecraft-header text-foreground"}>{name}</span>
                    <span className={"text-default-500"}>by {authors.join(", ")}</span>
                </div>
                <p className={"text-default-700 h-full"}>{description}</p>
                <div className={"flex flex-row gap-2 text-default-500 items-center"}>
                    <span className={"text-sm"}>Version: {version}</span>
                    <Divider orientation={"vertical"}/>
                    <span className={"text-sm"}>File: {filename}</span>
                    {(modrinthUpdateInfo || curseforgeUpdateInfo) && (
                        <>
                            <Divider orientation={"vertical"}/>
                            <span className={"text-sm text-warning"}>Update available</span>
                        </>
                    )}
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
                            target={"_self"}
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
                            target={"_self"}
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
                        isDisabled={isUpdating !== null}
                    >
                        <Icon icon={"pixelarticons:trash"}/>
                    </Button>
                </Tooltip>
            </div>
        </div>
    );
}
