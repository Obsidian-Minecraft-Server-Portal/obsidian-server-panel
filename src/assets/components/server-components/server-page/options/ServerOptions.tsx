import {Button, Divider, Input, Select, SelectItem, Switch, Textarea} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useCallback, useEffect, useRef, useState} from "react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import RamSlider from "../../RamSlider.tsx";
import JavaExecutableSelector from "../../JavaExecutableSelector.tsx";
import {Tooltip} from "../../../extended/Tooltip.tsx";

export function ServerOptions()
{
    const {server, updateServer, getEntries} = useServer();
    const [isLoading, setIsLoading] = useState(false);
    const [isSaving, setIsSaving] = useState(false);

    // Form state
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [javaExecutable, setJavaExecutable] = useState("");
    const [javaArgs, setJavaArgs] = useState("");
    const [minecraftArgs, setMinecraftArgs] = useState("");
    const [maxMemory, setMaxMemory] = useState(4);
    const [minMemory, setMinMemory] = useState(1);
    const [serverJar, setServerJar] = useState("");
    const [upnpEnabled, setUpnpEnabled] = useState(false);
    const [autoStart, setAutoStart] = useState(false);
    const [autoRestart, setAutoRestart] = useState(false);
    const firstLoadStateRef = useRef(false);

    // Available files
    const [availableFiles, setAvailableFiles] = useState<string[]>([]);


    const loadAvailableFiles = useCallback(async () =>
    {
        if (!server) return;

        setIsLoading(true);
        try
        {
            const entries = await getEntries("");
            const files = entries.entries
                .filter(entry => !entry.is_dir)
                .filter(entry =>
                {
                    const name = entry.filename.toLowerCase();
                    return name.endsWith(".jar") || name.endsWith(".bat") || name.endsWith(".sh");
                })
                .map(entry => entry.filename);

            setAvailableFiles(files);
        } catch (error)
        {
            console.error("Failed to load server files:", error);
        } finally
        {
            setIsLoading(false);
        }
    }, [server, getEntries]);

    const handleSave = useCallback(async () =>
    {
        if (!server) return;

        setIsSaving(true);
        try
        {
            await updateServer({
                name,
                description: description || null,
                java_executable: javaExecutable,
                java_args: javaArgs,
                minecraft_args: minecraftArgs,
                max_memory: maxMemory,
                min_memory: minMemory,
                server_jar: serverJar,
                upnp: upnpEnabled,
                auto_start: autoStart,
                auto_restart: autoRestart
            });
        } catch (error)
        {
            console.error("Failed to save server settings:", error);
        } finally
        {
            setIsSaving(false);
        }
    }, [
        server, updateServer, name, description, javaExecutable, javaArgs,
        minecraftArgs, maxMemory, minMemory, serverJar, upnpEnabled,
        autoStart, autoRestart
    ]);

    const hasChanges = useCallback(() =>
    {
        if (!server) return false;

        return (
            name !== server.name ||
            description !== (server.description || "") ||
            javaExecutable !== server.java_executable ||
            javaArgs !== server.java_args ||
            minecraftArgs !== server.minecraft_args ||
            maxMemory !== server.max_memory ||
            minMemory !== server.min_memory ||
            serverJar !== server.server_jar ||
            upnpEnabled !== server.upnp ||
            autoStart !== server.auto_start ||
            autoRestart !== server.auto_restart
        );
    }, [
        server, name, description, javaExecutable, javaArgs, minecraftArgs,
        maxMemory, minMemory, serverJar, upnpEnabled, autoStart, autoRestart
    ]);

    // Load server data when component mounts or server changes
    useEffect(() =>
    {
        // Only load data on first render or when server changes
        if (server)
        {
            if (!firstLoadStateRef.current)
            {
                firstLoadStateRef.current = true; // Prevent reloading on later renders
                setName(server.name);
                setDescription(server.description || "");
                setJavaExecutable(server.java_executable);
                setJavaArgs(server.java_args);
                setMinecraftArgs(server.minecraft_args);
                setMaxMemory(server.max_memory);
                setMinMemory(server.min_memory);
                setServerJar(server.server_jar);
                setUpnpEnabled(server.upnp);
                setAutoStart(server.auto_start);
                setAutoRestart(server.auto_restart);
            }

            // Load available server files
            loadAvailableFiles().then();
        }
    }, [server, loadAvailableFiles, firstLoadStateRef]);


    if (!server)
    {
        return (
            <div className="flex items-center justify-center h-full">
                <p className="text-gray-500 font-minecraft-body">No server selected</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-4 p-6 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] overflow-y-auto relative">
            <div className="flex items-center justify-between">
                <h2 className="text-xl font-minecraft-header">Server Configuration</h2>
                <Button
                    color="primary"
                    radius="none"
                    isLoading={isSaving}
                    isDisabled={!hasChanges()}
                    onPress={handleSave}
                    startContent={<Icon icon="pixelarticons:save"/>}
                    className={"data-[has-changes=true]:fixed data-[has-changes=false]:absolute data-[has-changes=false]:right-8 data-[has-changes=true]:right-16 z-10"}
                    data-has-changes={hasChanges()}
                >
                    Save Changes
                </Button>
            </div>

            {/* Basic Information */}
            <section className="flex flex-col gap-4">
                <h3 className="text-lg font-minecraft-header">Basic Information</h3>

                <Input
                    label="Server Name"
                    radius="none"
                    className="font-minecraft-body"
                    value={name}
                    onValueChange={setName}
                    startContent={<Icon icon="pixelarticons:device-game-console"/>}
                />

                <Textarea
                    label="Description"
                    radius="none"
                    className="font-minecraft-body"
                    placeholder="Optional server description..."
                    value={description}
                    onValueChange={setDescription}
                    startContent={<Icon icon="pixelarticons:note"/>}
                />
            </section>

            <Divider/>

            {/* Server Files */}
            <section className="flex flex-col gap-4">
                <div className="flex items-center gap-2">
                    <h3 className="text-lg font-minecraft-header">Server Files</h3>
                    <Tooltip content="Refresh file list">
                        <Button
                            isIconOnly
                            size="sm"
                            variant="light"
                            radius="none"
                            isLoading={isLoading}
                            onPress={loadAvailableFiles}
                        >
                            <Icon icon="pixelarticons:reload"/>
                        </Button>
                    </Tooltip>
                </div>

                <Select
                    label="Server JAR File"
                    radius="none"
                    disallowEmptySelection
                    className="font-minecraft-body"
                    selectedKeys={serverJar ? [serverJar] : []}
                    onSelectionChange={(keys) =>
                    {
                        const selected = Array.from(keys)[0] as string;
                        setServerJar(selected || "");
                    }}
                    startContent={<Icon icon="pixelarticons:file"/>}
                    placeholder="Select a server file..."
                    isLoading={isLoading}
                    classNames={{
                        base: "capitalize",
                        popoverContent: "rounded-none border-primary border-1"
                    }}

                    listboxProps={{
                        itemClasses: {
                            base: "rounded-none font-minecraft-body"
                        }
                    }}
                >
                    {availableFiles.map((file) => (
                        <SelectItem key={file}>
                            {file}
                        </SelectItem>
                    ))}
                </Select>
            </section>

            <Divider/>

            {/* Java Configuration */}
            <section className="flex flex-col gap-4">
                <h3 className="text-lg font-minecraft-header">Java Configuration</h3>

                <JavaExecutableSelector
                    defaultSelectedExecutable={javaExecutable}
                    onVersionChange={(executable) => setJavaExecutable(executable || "")}
                    isDisabled={false}
                />

                <Input
                    label="Additional Java Arguments"
                    radius="none"
                    className="font-minecraft-body"
                    placeholder="-XX:+UseG1GC -XX:+ParallelRefProcEnabled..."
                    value={javaArgs}
                    onValueChange={setJavaArgs}
                    startContent={<Icon icon="pixelarticons:terminal"/>}
                    description="Additional JVM arguments (excluding -Xmx and -Xms)"
                />
            </section>

            <Divider/>

            {/* Server Arguments */}
            <section className="flex flex-col gap-4">
                <h3 className="text-lg font-minecraft-header">Server Arguments</h3>

                <Input
                    label="Minecraft Server Arguments"
                    radius="none"
                    className="font-minecraft-body"
                    placeholder="--nogui --port 25565..."
                    value={minecraftArgs}
                    onValueChange={setMinecraftArgs}
                    startContent={<Icon icon="pixelarticons:command-line"/>}
                    description="Additional arguments passed to the Minecraft server"
                />
            </section>

            <Divider/>

            {/* Memory Configuration */}
            <section className="flex flex-col gap-4">
                <h3 className="text-lg font-minecraft-header">Memory Configuration</h3>

                <RamSlider
                    value={maxMemory}
                    onValueChange={setMaxMemory}
                    isDisabled={false}
                />

                <Input
                    label="Minimum Memory (GB)"
                    radius="none"
                    className="font-minecraft-body"
                    type="number"
                    min={1}
                    max={maxMemory}
                    value={minMemory.toString()}
                    onValueChange={(value) => setMinMemory(Number(value) || 1)}
                    startContent={<Icon icon="pixelarticons:memory"/>}
                    description="Minimum RAM allocation for the JVM"
                />
            </section>


            <Divider/>

            {/* Server Features */}
            <section className="flex flex-col gap-4">
                <h3 className="text-lg font-minecraft-header">Server Features</h3>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <Switch
                        isSelected={upnpEnabled}
                        onValueChange={setUpnpEnabled}
                        classNames={{
                            label: "font-minecraft-body"
                        }}
                    >
                        <div className="flex items-center gap-2">
                            <Icon icon="pixelarticons:wifi"/>
                            <span>Enable UPnP Port Forwarding</span>
                        </div>
                    </Switch>

                    <Switch
                        isSelected={autoStart}
                        onValueChange={setAutoStart}
                        classNames={{
                            label: "font-minecraft-body"
                        }}
                    >
                        <div className="flex items-center gap-2">
                            <Icon icon="pixelarticons:power"/>
                            <span>Auto-start on Boot</span>
                        </div>
                    </Switch>

                    <Switch
                        isSelected={autoRestart}
                        onValueChange={setAutoRestart}
                        classNames={{
                            label: "font-minecraft-body"
                        }}
                    >
                        <div className="flex items-center gap-2">
                            <Icon icon="pixelarticons:reload"/>
                            <span>Auto-restart on Crash</span>
                        </div>
                    </Switch>

                </div>
            </section>
        </div>
    );
}