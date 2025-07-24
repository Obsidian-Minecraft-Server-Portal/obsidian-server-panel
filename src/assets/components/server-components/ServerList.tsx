import {Button, Card, CardBody, CardHeader, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../extended/Tooltip.tsx";
import NewServerModal from "./NewServerModal.tsx";
import {useState} from "react";
import {useServer} from "../../providers/ServerProvider.tsx";
import {useMessage} from "../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../MessageModal.tsx";

export default function ServerList()
{
    const {servers} = useServer();
    const [isNewServerModalOpen, setIsNewServerModalOpen] = useState(false);
    return (
        <>
            <NewServerModal isOpen={isNewServerModalOpen} onClose={() => setIsNewServerModalOpen(false)}/>
            <Card radius={"none"} className={"w-full min-h-48"}>
                <CardHeader className={"font-minecraft-header flex flex-row w-full items-center justify-between"}>
                    <p className={"text-gray-400 text-sm"}>Servers</p>
                    <div className={"flex flex-row gap-2 items-center"}>
                        <Tooltip content={"Create a new server"}>
                            <Button isIconOnly radius={"none"} variant={"ghost"} color={"primary"} size={"sm"} onPress={() => setIsNewServerModalOpen(true)}><Icon icon={"pixelarticons:plus"} width={16}/></Button>
                        </Tooltip>
                    </div>
                </CardHeader>
                <CardBody className={"overflow-y-auto flex flex-col gap-2 grow shrink min-h-[200px] h-full"}>
                    {servers.length === 0 && (
                        <div className={"flex flex-col items-center justify-center h-full"}>
                            <p className={"text-gray-400/50 text-sm font-minecraft-body"}>No servers found. Create a new server to get started.</p>
                        </div>
                    )}
                    {servers.map((server) => (
                        <ServerItem key={server.id} serverId={server.id} serverName={server.name}/>
                    ))}
                </CardBody>
            </Card>
        </>
    );
}


type ServerItemProps = {
    serverId: string;
    serverName: string;
}

function ServerItem(props: ServerItemProps)
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