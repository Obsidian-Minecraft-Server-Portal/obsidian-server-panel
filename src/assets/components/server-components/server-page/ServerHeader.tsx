import {Button, cn, Divider, Image} from "@heroui/react";
import {Icon} from "@iconify-icon/react";

type ServerHeaderProps = {
    id: string,
    name: string,
    minecraft_version: string,
    server_type: string,
    loader_version: string,
    status: string
}

export function ServerHeader(props: ServerHeaderProps)
{
    const {id, name, minecraft_version, server_type, loader_version, status} = props;
    return (
        <div className={"flex flex-row gap-4 mt-8"}>
            <Image src={`/api/server/${id}/icon`}/>
            <div className={"flex flex-col gap-4"}>
                <h1 className={"text-4xl"}>{name}</h1>
                <div className={"flex flex-row gap-4 font-minecraft-body items-center"}>
                    <p className={"flex gap-2 p-2 hover:bg-default-50 transition-all duration-200 cursor-pointer"}><span className={"opacity-50 items-center flex gap-2"}> <Icon icon={"streamline:controller-1-remix"} className={"text-xl"}/>  Minecraft</span> <span className={"text-primary"}>{minecraft_version}</span></p>
                    {server_type.toLowerCase() !== "vanilla" && (
                        <>
                            <Divider orientation={"vertical"}/>

                            <p className={"flex gap-2 p-2 hover:bg-default-50 transition-all duration-200 cursor-pointer"}>
                                <span className={"opacity-50 items-center flex gap-2"}> <Icon icon={"pixelarticons:flatten"} className={"text-xl"}/> {server_type}</span>
                                <span className={"text-primary"}>{loader_version}</span>
                            </p>
                        </>
                    )}

                    <Divider orientation={"vertical"}/>
                    <p className={"flex gap-2 p-2 hover:bg-default-50 transition-all duration-200 cursor-pointer"}>
                        <span className={"opacity-50 items-center flex gap-2"}> <Icon icon={"pixelarticons:zap"} className={"text-xl"}/>  <span>Status</span> </span>
                        <span
                            className={
                                // "idle", "running", "stopped", "error", "starting", "stopping", "crashed", "hanging"
                                cn(
                                    "data-[status=idle]:text-warning data-[status=hanging]:text-warning",
                                    "data-[status=running]:text-success data-[status=starting]:text-success/90",
                                    "data-[status=stopped]:text-danger data-[status=stopping]:text-danger/90",
                                    "data-[status=error]:text-red-600 data-[status=crashed]:text-red-500"
                                )
                            }
                            data-status={status.toLowerCase()}
                        >
                                {status}
                            </span>
                    </p>
                </div>
            </div>
            <div className={"flex flex-row ml-auto items-center gap-4 font-minecraft-body"}>
                {
                    // "idle" | "running" | "stopped" | "error" | "starting" | "stopping" | "crashed" | "hanging"
                    (status.toLowerCase() === "idle" || status.toLowerCase() === "stopped" || status.toLowerCase() === "error" || status.toLowerCase() === "crashed") ? (
                        <Button radius={"none"} color={"primary"} variant={"ghost"} startContent={<Icon icon={"pixelarticons:play"} className={"text-xl"}/>}>Start Server</Button>
                    ) : status.toLowerCase() === "running" ? (
                        <>
                            <Button radius={"none"} color={"danger"} variant={"light"} startContent={<Icon icon={"pixelarticons:checkbox-on"} className={"text-xl"}/>}>Stop</Button>
                            <Button radius={"none"} variant={"solid"} startContent={<Icon icon={"pixelarticons:repeat"} className={"text-xl"}/>}>Restart</Button>
                        </>
                    ) : null}

            </div>

        </div>
    );
}