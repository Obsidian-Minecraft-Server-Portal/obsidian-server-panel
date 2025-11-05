import {Button, cn, Divider} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useServer} from "../../../providers/ServerProvider.tsx";
import {useEffect, useState} from "react";
import {useMessage} from "../../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../../MessageModal.tsx";
import {motion} from "framer-motion";
import {ServerIcon} from "./ServerIcon.tsx";
import ReactMarkdown from "react-markdown";
import $ from "jquery";
import {Tooltip} from "../../extended/Tooltip.tsx";

type ServerHeaderProps = {
    id: string,
    name: string,
    description: string,
    minecraft_version: string,
    server_type: string,
    loader_version: string,
    status: string
}

type Player =
    {
        name: string;
        id: string;
    }

type PingResponse = {
    sample?: Player[];
    online_players?: number;
    max_players?: number;
}

export function ServerHeader(props: ServerHeaderProps)
{
    const {id, name, description, minecraft_version, server_type, loader_version, status} = props;
    const {startServer, stopServer, restartServer, killServer, isServerRunning} = useServer();
    const [isServerStarting, setIsServerStarting] = useState<boolean>(false);
    const [ping, setPing] = useState<PingResponse>();
    const {open} = useMessage();

    useEffect(() =>
    {
        if (status.toLowerCase() !== "running") {
            setPing(undefined);
            return;
        }

        // Fetch ping once when server starts running
        const fetchPing = async () => {
            try
            {
                const pingResponse: PingResponse = await $.get(`/api/server/${id}/ping`);
                setPing(pingResponse);
            } catch
            {
                // If the ping fails, we assume the server is not running
                setPing(undefined);
            }
        };

        fetchPing();

        // Listen for server ping updates via WebSocket (if implemented)
        const handleServerPing = (event: Event) => {
            const customEvent = event as CustomEvent;
            const {serverId, ping: pingData} = customEvent.detail;
            if (serverId === id) {
                setPing(pingData);
            }
        };

        window.addEventListener('server-ping', handleServerPing);

        return () => {
            window.removeEventListener('server-ping', handleServerPing);
        };
    }, [status, id]);
    return (
        <div className={"flex flex-row gap-4 mt-8"}>
            <motion.div
                initial={{opacity: 0, x: -20}}
                animate={{opacity: 1, x: 0}}
                exit={{opacity: 0, x: -40}}
                transition={{duration: 0.3, ease: "easeInOut"}}
                className={"flex flex-row gap-4 mt-8"}
            >
                <ServerIcon id={id} isChangeEnabled={true} size={"md"}/>
                <div className={"flex flex-col gap-4"}>
                    <div className={"flex flex-col gap-1 h-full"}>
                        <h1 className={"text-4xl"}>{name}</h1>
                        {description && (
                            <div className={"text-normal text-gray-500 font-minecraft-body"}>
                                <ReactMarkdown
                                    components={{
                                        // Disable heading size changes - keep same font size
                                        h1: ({children}) => <span className="font-bold">{children}</span>,
                                        h2: ({children}) => <span className="font-bold">{children}</span>,
                                        h3: ({children}) => <span className="font-bold">{children}</span>,
                                        h4: ({children}) => <span className="font-bold">{children}</span>,
                                        h5: ({children}) => <span className="font-bold">{children}</span>,
                                        h6: ({children}) => <span className="font-bold">{children}</span>,
                                        // Style links appropriately
                                        a: ({href, children}) => (
                                            <a
                                                href={href}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="text-primary hover:text-primary-600 underline transition-colors"
                                            >
                                                {children}
                                            </a>
                                        ),
                                        // Keep text styling
                                        strong: ({children}) => <strong className="font-bold">{children}</strong>,
                                        em: ({children}) => <em className="italic">{children}</em>,
                                        // Style inline code
                                        code: ({children}) => (
                                            <code className="bg-default-100 px-1 py-0.5 rounded text-sm font-mono">
                                                {children}
                                            </code>
                                        ),
                                        // Remove paragraph margins to keep inline
                                        p: ({children}) => <span>{children}</span>,
                                        // Style strikethrough
                                        del: ({children}) => <del className="line-through">{children}</del>
                                    }}
                                    disallowedElements={["img"]} // Disable images for security
                                >
                                    {description}
                                </ReactMarkdown>
                            </div>
                        )}
                    </div>
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
                                        "data-[status=error]:text-red-600 data-[status=crashed]:text-red-500",
                                        "capitalize"
                                    )
                                }
                                data-status={isServerStarting ? "starting" : status.toLowerCase()}
                            >
                                {isServerStarting ? "Starting" : status}
                            </span>
                        </p>
                        {isServerRunning() && ping &&
                            <>
                                <Divider orientation={"vertical"}/>
                                <Tooltip
                                    content={
                                        <div className={"flex flex-col gap-2"}>
                                            <p className={"text-sm text-gray-500"}>Online Players: {ping.online_players ?? "Unknown"}</p>
                                            {ping.sample?.map(player => (
                                                <p key={player.id} className={"text-sm text-gray-500"}>{player.name}</p>
                                            ))}
                                        </div>
                                    }
                                >
                                    <p className={"flex gap-2 p-2 hover:bg-default-50 transition-all duration-200 cursor-pointer"}>
                                        <span className={"opacity-50 items-center flex gap-2"}> <Icon icon={"pixelarticons:users"} className={"text-xl"}/>  <span>Players</span> </span>
                                        <span className={"capitalize"}>{ping.online_players}</span>
                                    </p>
                                </Tooltip>
                            </>
                        }
                    </div>
                </div>
            </motion.div>
            <motion.div
                className={"flex flex-row ml-auto items-center gap-4 font-minecraft-body"}
                initial={{opacity: 0, x: 20}}
                animate={{opacity: 1, x: 0}}
                exit={{opacity: 0, x: 40}}
                transition={{duration: 0.3, ease: "easeInOut"}}
            >
                {
                    // "idle" | "running" | "stopped" | "error" | "starting" | "stopping" | "crashed" | "hanging"
                    (status.toLowerCase() === "idle" || status.toLowerCase() === "stopped" || status.toLowerCase() === "error" || status.toLowerCase() === "crashed") ? (
                        <Button radius={"none"} color={"primary"} variant={"solid"} isLoading={isServerStarting} startContent={<Icon icon={"pixelarticons:play"} className={"text-xl"}/>} onPress={async () =>
                        {
                            setIsServerStarting(true);
                            await startServer(id);
                            setIsServerStarting(false);
                        }}>Start</Button>
                    ) : status.toLowerCase() === "running" ? (
                        <>
                            <Button radius={"none"} color={"danger"} variant={"light"} startContent={<Icon icon={"pixelarticons:checkbox-on"} className={"text-xl"}/>} onPress={() => stopServer(id)}>Stop</Button>
                            <Button radius={"none"} variant={"solid"} startContent={<Icon icon={"pixelarticons:repeat"} className={"text-xl"}/>} onPress={() => restartServer(id)}>Restart</Button>
                        </>
                    ) : status === "stopping" ? (
                        <Button radius={"none"} color={"danger"} variant={"light"} startContent={<Icon icon={"tabler:cancel"} className={"text-xl"}/>} onPress={async () =>
                        {
                            const response = await open({
                                title: "Kill Server",
                                body: "Are you sure you want to kill the server? This will forcefully stop the server and may cause data loss.",
                                severity: "danger",
                                responseType: MessageResponseType.YesNo
                            });
                            if (response) await killServer();
                        }}>Kill</Button>
                    ) : null}

            </motion.div>

        </div>
    );
}
