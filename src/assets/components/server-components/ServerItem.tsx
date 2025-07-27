import {useServer} from "../../providers/ServerProvider.tsx";
import {useMessage} from "../../providers/MessageProvider.tsx";
import {Button, Link} from "@heroui/react";
import {Tooltip} from "../extended/Tooltip.tsx";
import {MessageResponseType} from "../MessageModal.tsx";
import {Icon} from "@iconify-icon/react";

type ServerItemProps = {
    serverId: string;
    serverName: string;
}

export function ServerItem(props: ServerItemProps)
{
    const {serverId, serverName} = props;
    const {deleteServer} = useServer();
    const {open} = useMessage();

    return (
        <div className={"flex flex-col gap-2"}>
            <div className={"flex flex-row items-center justify-between p-2 hover:bg-gray-600/20 transition-all duration-200"}>
                <Link href={`/app/servers/${serverId}`} className={"text-gray-400 text-sm cursor-pointer w-full py-2 font-minecraft-body"}>{serverName}</Link>
                <div className={"flex flex-row gap-2 items-center"}>
                    <Tooltip content={"Edit server"}>
                        <Button
                            isIconOnly
                            radius={"none"}
                            variant={"ghost"}
                            size={"sm"}
                            onPress={async () =>
                            {
                                await open({
                                    title: "Edit Server",
                                    body: "This is not implemented yet.",
                                    responseType: MessageResponseType.Close,
                                    severity: "warning"
                                });
                            }}
                        >
                            <Icon icon={"pixelarticons:edit"} width={16}/>
                        </Button>
                    </Tooltip>
                    <Tooltip content={"Delete server"}>
                        <Button
                            isIconOnly
                            radius={"none"}
                            variant={"ghost"}
                            color={"danger"}
                            size={"sm"}
                            onPress={async () =>
                            {
                                let shouldDelete = await open({
                                    title: "Delete Server",
                                    body: <p>Are you sure you want to delete the server <span className={"text-danger font-bold underline"}>{serverName}</span>? This action cannot be undone.</p>,
                                    responseType: MessageResponseType.OkayCancel,
                                    severity: "danger"
                                });
                                if (shouldDelete)
                                {
                                    await deleteServer(serverId);
                                }
                            }}
                        >
                            <Icon icon={"pixelarticons:trash"} width={16}/>
                        </Button>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
}