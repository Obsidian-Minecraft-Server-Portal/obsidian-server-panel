import {Button, Card, CardBody, CardHeader} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../extended/Tooltip.tsx";
import NewServerModal from "./NewServerModal.tsx";
import {useEffect, useState} from "react";
import {useServer} from "../../providers/ServerProvider.tsx";
import {ServerItem} from "./ServerItem.tsx";

export default function ServerList()
{
    const {servers, loadServers} = useServer();
    const [isNewServerModalOpen, setIsNewServerModalOpen] = useState(false);

    useEffect(() =>
    {
        // Load servers once on initial mount
        // WebSocket updates are handled by ServerProvider
        loadServers().catch(console.error);
    }, [loadServers]);

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