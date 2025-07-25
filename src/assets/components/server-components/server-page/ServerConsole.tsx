import {LogView} from "./LogView.tsx";
import {Autocomplete, AutocompleteItem, Button, Input} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../extended/Tooltip.tsx";
import {useServer} from "../../../providers/ServerProvider.tsx";
import {useCallback, useEffect, useState} from "react";


type ServerOverviewProps = {
    id: string;
}

export default function ServerConsole(props: ServerOverviewProps)
{
    const {id} = props;
    const {servers, subscribeToConsole, getLogs, getLog} = useServer();
    const [log, setLog] = useState("");
    const [logFiles, setLogFiles] = useState<string[]>([]);
    const [selectedLogFile, setSelectedLogFile] = useState("latest.log");
    const [isRunning, setIsRunning] = useState(false);
    const [isAutoscrollEnabled, setIsAutoscrollEnabled] = useState(true);

    const isAtBottom = useCallback(() =>
    {
        const logView = document.querySelector("#log-view");
        if (!logView) return false;

        // Consider "at bottom" if within 5 pixels of the bottom
        const threshold = 5;
        return logView.scrollTop + logView.clientHeight >= logView.scrollHeight - threshold;
    }, []);

    const handleScroll = useCallback(() =>
    {
        if (isAtBottom())
        {
            setIsAutoscrollEnabled(true);
        } else
        {
            setIsAutoscrollEnabled(false);
        }
    }, [isAtBottom]);

    const scrollToBottom = useCallback(() =>
    {
        const logView = document.querySelector("#log-view");
        if (logView)
        {
            logView.scrollTop = logView.scrollHeight;
            setIsAutoscrollEnabled(true);
        }
    }, []);

    useEffect(() =>
    {
        const server = servers.find(s => s.id === id);
        if (!server)
        {
            console.error(`Server with id ${id} not found.`);
            return;
        }
        let status = server.status.toLowerCase();
        const isRunning = status === "running" || status === "starting" || status === "stopping" || status === "hanging";
        setIsRunning(isRunning);

        if (!isRunning)
        {
            // Fetch log files
            getLogs(server.id)
                .then(logFiles => [...logFiles].sort((a, b) => (a === "latest.log" ? -1 : b === "latest.log" ? 1 : 0)))
                .then(async logs =>
                {
                    if (logs.includes("latest.log"))
                    {
                        setSelectedLogFile("latest.log");
                        try
                        {
                            const logContent = await getLog("latest.log", server.id);
                            setLog(logContent);
                        } catch (e)
                        {
                            console.error(`Failed to fetch log latest.log for server ${server.id}:`, e);
                            setLog("");
                        }
                    }
                    setLogFiles(logs);
                });
        } else
        {
            (async () =>
            {
                try
                {
                    const logContent = await getLog("latest.log", server.id);
                    setLog(logContent.split("\n").slice(-100).join("\n")); // Fetch only the last 100 lines for performance
                } catch (e)
                {
                    console.error(`Failed to fetch log latest.log for server ${server.id}:`, e);
                    setLog("");
                }

                subscribeToConsole((newLog) =>
                {
                    setLog(prev => prev + newLog);
                    // Only autoscroll if autoscroll is enabled
                    if (isAutoscrollEnabled)
                    {
                        scrollToBottom();
                    }
                }, server.id);
            })();
        }

    }, [servers, id, isAutoscrollEnabled, scrollToBottom]);

    // Add scroll event listener
    useEffect(() =>
    {
        const logView = document.querySelector("#log-view");
        if (logView)
        {
            logView.addEventListener("scroll", handleScroll);
            return () =>
            {
                logView.removeEventListener("scroll", handleScroll);
            };
        }
    }, [handleScroll]);

    return (
        <div className={"flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative"}>
            <div className={"flex flex-row justify-between"}>
                <h1 className={"text-4xl mb-4"}>Console</h1>
                {!isRunning && (

                    <Autocomplete
                        placeholder={"Search logs..."}
                        className={"w-1/3 font-minecraft-body"}
                        radius={"none"}
                        selectedKey={selectedLogFile}
                        onSelectionChange={async (value) =>
                        {
                            setSelectedLogFile(value as string);
                            const server = servers.find(s => s.id === id);
                            if (!server) return;
                            try
                            {
                                const logContent = await getLog(value as string, server.id);
                                setLog(logContent);
                            } catch (e)
                            {
                                console.error(`Failed to fetch log ${value} for server ${server.id}:`, e);
                                setLog("");
                            }
                        }}
                        classNames={{
                            popoverContent: "rounded-none border-primary border-1"
                        }}
                        endContent={
                            <Tooltip content={"Download Log"}>
                                <Button isIconOnly variant={"light"} size={"sm"} radius={"none"}>
                                    <Icon icon={"pixelarticons:flatten"}/>
                                </Button>
                            </Tooltip>
                        }
                    >
                        {logFiles.map((file) => (
                            <AutocompleteItem key={file} textValue={file}>{file}</AutocompleteItem>
                        ))}
                    </Autocomplete>
                )}
            </div>
            <LogView log={log}/>
            <Tooltip content={"Scroll to bottom"}>
                <Button isIconOnly size={"sm"} radius={"none"} className={"absolute bottom-8 right-8 text-xl"} onPress={scrollToBottom}><Icon icon={"pixelarticons:arrow-down"}/></Button>
            </Tooltip>
            {isRunning && <CommandInput id={id}/>}
        </div>
    );
}


function CommandInput({id}: { id: string })
{
    const {sendCommand} = useServer();
    const [value, setValue] = useState("");

    const handleCommandSubmit = async () =>
    {
        if (value.trim() === "") return;
        try
        {
            await sendCommand(value.trim(), id);
            setValue(""); // Clear the input after sending the command
        } catch (error)
        {
            console.error("Failed to send command:", error);
        }
    };

    return (

        <div className={"absolute bottom-8 left-8 right-8 font-minecraft-body"}>
            <Input
                placeholder={"Send a command..."}
                radius={"none"}
                value={value}
                onValueChange={setValue}
                startContent={<Icon icon={"mdi:console"}/>}
                endContent={
                    <Tooltip content={"Send Command"}>
                        <Button isIconOnly variant={"light"} size={"sm"} radius={"none"} onPress={handleCommandSubmit}><Icon icon={"mdi:send"}/></Button>
                    </Tooltip>
                }
                onKeyUp={async (e) =>
                {
                    if (e.key === "Enter" && e.currentTarget.value.trim() !== "") await handleCommandSubmit();
                }}
            />
        </div>
    );
}