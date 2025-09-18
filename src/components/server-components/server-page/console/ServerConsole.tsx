import {LogView} from "./LogView.tsx";
import {Autocomplete, AutocompleteItem, Button} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useCallback, useEffect, useState} from "react";
import {CommandInput} from "./CommandInput.tsx";


export default function ServerConsole()
{
    const {server, isServerRunning, subscribeToConsole, hasActiveConsoleConnection, getLogs, getLog, downloadEntry} = useServer();
    const [log, setLog] = useState<string[]>([]);
    const [logFiles, setLogFiles] = useState<string[]>([]);
    const [selectedLogFile, setSelectedLogFile] = useState("latest.log");
    const [isAutoscrollEnabled, setIsAutoscrollEnabled] = useState(true);
    const [isRunning, setIsRunning] = useState(isServerRunning());

    const scrollToBottom = useCallback(() =>
    {
        const logView = document.querySelector("#log-view");
        if (logView)
        {
            logView.scrollTop = logView.scrollHeight;
        }
        setIsAutoscrollEnabled(true);
    }, [setIsAutoscrollEnabled]);

    const handleScroll = useCallback(() =>
    {
        const logView = document.querySelector("#log-view");
        if (logView)
        {
            const {scrollTop, scrollHeight, clientHeight} = logView;
            const isAtBottom = scrollTop + clientHeight >= scrollHeight - 10; // 10px threshold

            if (isAtBottom && !isAutoscrollEnabled)
            {
                setIsAutoscrollEnabled(true);
            } else if (!isAtBottom && isAutoscrollEnabled)
            {
                setIsAutoscrollEnabled(false);
            }
        }
    }, [isAutoscrollEnabled]);

    const downloadSelectedLogFile = useCallback(async () =>
    {
        await downloadEntry(`logs/${selectedLogFile}`);
    }, [selectedLogFile]);


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

    useEffect(() =>
    {
        if (!server) return;
        let running = isServerRunning();
        setIsRunning(running);
        if (running)
        {
            // load the latest log file when the server is running
            getLog("latest.log")
                .then((logContent) => setLog(logContent.split("\n").splice(-1000)))
                .catch(() => setLog(["Failed to load latest log file. Please try again."]))
                .finally(() =>
                {
                    if (isAutoscrollEnabled)
                    {
                        setTimeout(() =>
                        {
                            scrollToBottom();
                        }, 0);
                    }
                });
        }
    }, [server]);

    useEffect(() =>
    {
        if (isRunning)
        {
            if (hasActiveConsoleConnection())
            {
                console.warn("Already subscribed to console, skipping subscription.");
                return;
            }
            console.log("Subscribing to console for server:", server?.id);

            setLog([])
            subscribeToConsole((newLog) =>
            {
                setLog((prevLog) =>
                {
                    const updatedLog = [...prevLog, newLog].slice(-1000); // Keep the last 1000 lines
                    if (isAutoscrollEnabled)
                    {
                        setTimeout(() =>
                        {
                            scrollToBottom();
                        }, 0);
                    }
                    return updatedLog;
                });
            });
        } else
        {
            getLogs().then((logFiles) => setLogFiles(logFiles.sort((a, b) => a === "latest.log" ? -1 : b === "latest.log" ? 1 : 0))).finally(async () =>
            {
                const logContent = await getLog("latest.log");
                setLog(logContent.split("\n"));
            });
        }
    }, [isRunning]);

    if (!server) return null;
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
                            try
                            {
                                const logContent = await getLog(value as string, server.id);
                                setLog(logContent.split("\n"));
                            } catch (e)
                            {
                                console.error(`Failed to fetch log ${value} for server ${server.id}:`, e);
                                setLog(["Failed to fetch log. Please try again."]);
                            }
                        }}
                        classNames={{
                            popoverContent: "rounded-none border-primary border-1"
                        }}
                        endContent={
                            <Tooltip content={"Download Log"}>
                                <Button isIconOnly variant={"light"} size={"sm"} radius={"none"} onPress={downloadSelectedLogFile}>
                                    <Icon icon={"pixelarticons:flatten"}/>
                                </Button>
                            </Tooltip>
                        }
                        listboxProps={{
                            itemClasses: {
                                base: "rounded-none font-minecraft-body"
                            }
                        }}
                    >
                        {logFiles.map((file) => (
                            <AutocompleteItem key={file} textValue={file}>{file}</AutocompleteItem>
                        ))}
                    </Autocomplete>
                )}
            </div>
            <LogView log={log}/>
            <Tooltip content={"Scroll to bottom"}>
                <Button
                    isIconOnly
                    size={"sm"}
                    radius={"none"}
                    className={`absolute data-[running=true]:bottom-24 bottom-8 right-8 text-xl ${!isAutoscrollEnabled ? "opacity-100" : "opacity-50"}`}
                    onPress={scrollToBottom}
                    data-running={isRunning}
                    color={!isAutoscrollEnabled ? "primary" : "default"}
                >
                    <Icon icon={"pixelarticons:arrow-down"}/>
                </Button>
            </Tooltip>
            {isRunning && <CommandInput/>}
        </div>
    );
}