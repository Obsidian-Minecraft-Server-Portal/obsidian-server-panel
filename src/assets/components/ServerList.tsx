import {Button, Card, CardBody, CardHeader, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "./extended/Tooltip.tsx";

export default function ServerList()
{
    return (
        <Card radius={"none"} className={"w-full min-h-48"}>
            <CardHeader className={"font-minecraft-header flex flex-row w-full items-center justify-between"}>
                <p className={"text-gray-400 text-sm"}>Servers</p>
                <div className={"flex flex-row gap-2 items-center"}>
                    <Tooltip content={"Create a new server"}>
                        <Button isIconOnly radius={"none"} variant={"ghost"} color={"primary"} size={"sm"}><Icon icon={"pixelarticons:plus"} width={16}/></Button>
                    </Tooltip>
                </div>
            </CardHeader>
            <CardBody className={"overflow-y-auto flex flex-col gap-2 grow shrink min-h-[200px] max-h-[800px] h-[calc(100dvh_-_308px)]"}>
                {Array.from({length: 50}).map((_, i) => (
                    <ServerItem key={i} serverId={i.toString(36)} serverName={`Server ${i + 1}`} />
                ))}
            </CardBody>
        </Card>
    );
}


type ServerItemProps = {
    serverId: string;
    serverName: string;
}
function ServerItem(props: ServerItemProps){
    const {serverId, serverName} = props;

    return (
        <div className={"flex flex-col gap-2"}>
            <div className={"flex flex-row items-center justify-between p-2 hover:bg-gray-600/20 transition-all duration-200"}>
                <Link href={`/app/servers/${serverId}`} className={"text-gray-400 text-sm cursor-pointer w-full py-2"}>{serverName}</Link>
                <div className={"flex flex-row gap-2 items-center"}>
                    <Tooltip content={"Edit server"}>
                        <Button isIconOnly radius={"none"} variant={"ghost"} size={"sm"}><Icon icon={"pixelarticons:edit"} width={16}/></Button>
                    </Tooltip>
                    <Tooltip content={"Delete server"}>
                        <Button isIconOnly radius={"none"} variant={"ghost"} color={"danger"} size={"sm"}><Icon icon={"pixelarticons:trash"} width={16}/></Button>
                    </Tooltip>
                </div>
            </div>
        </div>
    )
}