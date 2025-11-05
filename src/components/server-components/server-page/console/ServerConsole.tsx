import {LogView} from "./LogView.tsx";
import {Autocomplete, AutocompleteItem, Button} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useCallback, useEffect, useRef, useState} from "react";
import {CommandInput} from "./CommandInput.tsx";


export default function ServerConsole()
{
    const {server, isServerRunning, subscribeToConsole, getLogs, getLog, downloadEntry} = useServer();
    const [log, setLog] = useState<string[]>([]);
    const [logFiles, setLogFiles] = useState<string[]>([]);
    const [selectedLogFile, setSelectedLogFile] = useState("latest.log");
    const [isAutoscrollEnabled, setIsAutoscrollEnabled] = useState(true);
    const [isRunning, setIsRunning] = useState(isServerRunning());
    const logViewRef = useRef<HTMLDivElement>(null);
    const isAutoscrollEnabledRef = useRef(true);

    // Keep ref in sync with state
    useEffect(() => {
        isAutoscrollEnabledRef.current = isAutoscrollEnabled;
    }, [isAutoscrollEnabled]);

    const scrollToBottom = useCallback(() =>
    {
        if (logViewRef.current)
        {
            logViewRef.current.scrollTop = logViewRef.current.scrollHeight;
            setIsAutoscrollEnabled(true);
        }
    }, []);

    const handleScroll = useCallback(() =>
    {
        if (logViewRef.current)
        {
            const {scrollTop, scrollHeight, clientHeight} = logViewRef.current;
            const isAtBottom = scrollTop + clientHeight >= scrollHeight - 10; // 10px threshold

            if (isAtBottom && !isAutoscrollEnabledRef.current)
            {
                setIsAutoscrollEnabled(true);
            } else if (!isAtBottom && isAutoscrollEnabledRef.current)
            {
                setIsAutoscrollEnabled(false);
            }
        }
    }, []);

    const downloadSelectedLogFile = useCallback(async () =>
    {
        await downloadEntry(`logs/${selectedLogFile}`);
    }, [selectedLogFile]);


    useEffect(() =>
    {
        const logView = logViewRef.current;
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
            // Switch to live log when server starts running
            setSelectedLogFile("live-log");
            // load the latest log file when the server is running
            getLog("latest.log")
                .then((logContent) => setLog(logContent.split("\n").splice(-1000)))
                .catch(() => setLog(["Failed to load latest log file. Please try again."]))
                .finally(() =>
                {
                    if (isAutoscrollEnabledRef.current)
                    {
                        requestAnimationFrame(() => {
                            scrollToBottom();
                        });
                    }
                });
        }
    }, [server, isServerRunning, getLog, scrollToBottom]);

    useEffect(() =>
    {
        if (isRunning)
        {
            // Add "live-log" option to the log files list
            setLogFiles(["live-log"]);

            console.log("Subscribing to console for server:", server?.id);

            setLog([])
            const cleanup = subscribeToConsole((newLog) =>
            {
                setLog((prevLog) =>
                {
                    const updatedLog = [...prevLog, newLog].slice(-1000); // Keep the last 1000 lines
                    if (isAutoscrollEnabledRef.current)
                    {
                        requestAnimationFrame(() => {
                            if (logViewRef.current)
                            {
                                logViewRef.current.scrollTop = logViewRef.current.scrollHeight;
                            }
                        });
                    }
                    return updatedLog;
                });
            });

            // Return cleanup function to be called when component unmounts or isRunning changes
            return cleanup;
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
                <Autocomplete
                    placeholder={"Search logs..."}
                    className={"w-1/3 font-minecraft-body"}
                    radius={"none"}
                    selectedKey={selectedLogFile}
                    onSelectionChange={async (value) =>
                    {
                        const selectedValue = value as string;
                        setSelectedLogFile(selectedValue);

                        // Don't fetch if it's the live log (already streaming via SSE)
                        if (selectedValue === "live-log") {
                            return;
                        }

                        try
                        {
                            const logContent = await getLog(selectedValue, server.id);
                            setLog(logContent.split("\n"));
                        } catch (e)
                        {
                            console.error(`Failed to fetch log ${selectedValue} for server ${server.id}:`, e);
                            setLog(["Failed to fetch log. Please try again."]);
                        }
                    }}
                    classNames={{
                        popoverContent: "rounded-none border-primary border-1"
                    }}
                    endContent={
                        selectedLogFile !== "live-log" && (
                            <Tooltip content={"Download Log"}>
                                <Button isIconOnly variant={"light"} size={"sm"} radius={"none"} onPress={downloadSelectedLogFile}>
                                    <Icon icon={"pixelarticons:flatten"}/>
                                </Button>
                            </Tooltip>
                        )
                    }
                    listboxProps={{
                        itemClasses: {
                            base: "rounded-none font-minecraft-body"
                        }
                    }}
                >
                    {logFiles.map((file) => (
                        <AutocompleteItem key={file} textValue={file === "live-log" ? "Live Log" : file}>
                            {file === "live-log" ? "Live Log" : file}
                        </AutocompleteItem>
                    ))}
                </Autocomplete>
            </div>
            <LogView ref={logViewRef} log={log}/>
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