import "../../../../ts/string-ext.ts";
import {Divider, Input, Select, SelectItem, Switch, Tab, Tabs, Textarea} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useCallback, useEffect, useRef, useState} from "react";
import {LoaderType, useServer} from "../../../../providers/ServerProvider.tsx";
import RamSlider from "../../RamSlider.tsx";
import JavaExecutableSelector from "../../JavaExecutableSelector.tsx";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {MinecraftVersionSelector} from "../../version-selectors/MinecraftVersionSelector.tsx";
import {ForgeVersionSelector} from "../../version-selectors/ForgeVersionSelector.tsx";
import {FabricVersionSelector} from "../../version-selectors/FabricVersionSelector.tsx";
import {QuiltVersionSelector} from "../../version-selectors/QuiltVersionSelector.tsx";
import {NeoForgeVersionSelector} from "../../version-selectors/NeoForgeVersionSelector.tsx";
import {FileInput} from "../../../extended/FileInput.tsx";
import {NeoForge} from "../../../icons/NeoForge.svg.tsx";
import Quilt from "../../../icons/Quilt.svg.tsx";
import {getMinecraftVersionDownloadUrl} from "../../../../ts/minecraft-versions.ts";
import {ServerIcon} from "../ServerIcon.tsx";
import {Button} from "../../../extended/Button.tsx";

export function ServerOptions()
{
    const {server, updateServer, getEntries, uploadFromUrl, uploadFile, loadServer} = useServer();
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

    // Loader configuration state
    const [loaderType, setLoaderType] = useState<LoaderType>("vanilla");
    const [minecraftVersion, setMinecraftVersion] = useState("");
    const [loaderVersion, setLoaderVersion] = useState("");
    const [loaderUrl, setLoaderUrl] = useState<string | undefined>(undefined);
    const [customJarFile, setCustomJarFile] = useState<File | undefined>(undefined);
    const [isUploadingLoader, setIsUploadingLoader] = useState(false);
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

    const handleLoaderChange = useCallback((url: string | undefined, version: string | undefined) =>
    {
        if (!version) return;
        setLoaderUrl(url);
        setLoaderVersion(version);
    }, []);

    const generateNewJarFilename = useCallback(() =>
    {
        let filepath = `server-${minecraftVersion}.jar`;
        if (loaderType.equalsIgnoreCase("fabric") || loaderType.equalsIgnoreCase("quilt") || loaderType.equalsIgnoreCase("neoforge"))
        {
            filepath = `${loaderType}-${loaderVersion}-${minecraftVersion}-server.jar`;
        } else if (loaderType.equalsIgnoreCase("forge"))
        {
            // Forge uses installer JARs that need to be run to generate the actual server JAR
            filepath = `forge-${loaderVersion}-installer.jar`;
        } else if (loaderType.equalsIgnoreCase("custom"))
        {
            filepath = `custom-${minecraftVersion}.jar`;
        }
        return filepath.toLowerCase();

    }, [loaderType, loaderVersion, minecraftVersion]);

    const hasLoaderChanges = useCallback(() =>
    {
        if (!server) return false;
        return (
            loaderType.toLowerCase() !== server.server_type.toLowerCase() ||
            minecraftVersion !== server.minecraft_version ||
            loaderVersion !== server.loader_version
        );
    }, [server, loaderType, minecraftVersion, loaderVersion]);

    const installSelectedLoader = useCallback(async () =>
    {
        if (!server) return;
        setIsUploadingLoader(true);
        const newJarFilename = generateNewJarFilename();
        console.log("Uploading new server jar:", newJarFilename, "Loader: ", loaderType, "Version: ", minecraftVersion, "Loader URL: ", loaderUrl, "Custom Jar: ", customJarFile);

        if (loaderType !== "custom")
        {
            if (!loaderUrl)
            {
                throw new Error(`Loader URL is not defined for selected loader: ${loaderType}`);
            }


            await uploadFromUrl(
                loaderType.equalsIgnoreCase("vanilla") ? await getMinecraftVersionDownloadUrl(minecraftVersion) : loaderUrl,
                newJarFilename,
                (progress) => console.log(`Downloading ${loaderType} server: ${progress}%`),
                () => console.log("Download complete"),
                (error) => console.error("Error uploading server jar:", error),
                server.id
            );
        } else
        {
            if (!customJarFile)
            {
                throw new Error("Please select a custom jar file.");
            }

            await uploadFile(
                customJarFile,
                newJarFilename,
                (bytes) => console.log(`Uploading custom jar: ${bytes} bytes`),
                () => console.log("Upload cancelled"),
                server.id
            );
        }

        setIsUploadingLoader(false);
    }, [server, loaderType, minecraftVersion, loaderUrl, customJarFile, uploadFromUrl, uploadFile, generateNewJarFilename]);

    const handleSave = useCallback(async () =>
    {
        if (!server) return;

        setIsSaving(true);

        try
        {
            let finalServerJar = serverJar;

            // If loader configuration changed, upload a new server jar
            if (hasLoaderChanges())
            {
                await installSelectedLoader();
            }

            // Set isUploadingLoader to false only after upload is complete
            setIsUploadingLoader(false);

            await updateServer({
                name,
                description: description || null,
                java_executable: javaExecutable,
                java_args: javaArgs,
                minecraft_args: minecraftArgs,
                max_memory: maxMemory,
                min_memory: minMemory,
                server_jar: finalServerJar,
                upnp: upnpEnabled,
                auto_start: autoStart,
                auto_restart: autoRestart,
                // Update loader configuration
                server_type: loaderType,
                minecraft_version: minecraftVersion,
                loader_version: loaderVersion
            });

            // Refresh file list after potential jar upload
            if (hasLoaderChanges())
            {
                await loadAvailableFiles();
            }
        } catch (error)
        {
            console.error("Failed to save server settings:", error);
        } finally
        {
            setIsSaving(false);
            setIsUploadingLoader(false);
            await loadServer(server.id);
        }
    }, [
        server, updateServer, name, description, javaExecutable, javaArgs,
        minecraftArgs, maxMemory, minMemory, serverJar, upnpEnabled,
        autoStart, autoRestart, loaderType, minecraftVersion, loaderVersion, hasLoaderChanges,
        generateNewJarFilename, loaderUrl, customJarFile, uploadFromUrl,
        uploadFile, loadAvailableFiles
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
            autoRestart !== server.auto_restart ||
            hasLoaderChanges()
        );
    }, [
        server, name, description, javaExecutable, javaArgs, minecraftArgs,
        maxMemory, minMemory, serverJar, upnpEnabled, autoStart, autoRestart,
        hasLoaderChanges
    ]);
    // Load server data when the component mounts or server changes
    useEffect(() =>
    {
        if (server)
        {
            // Only load data once when the server is first set or when server ID changes
            if (!firstLoadStateRef.current)
            {
                console.log("Options Server", server);
                firstLoadStateRef.current = true;
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

                // Load loader configuration
                setLoaderType(server.server_type);
                setMinecraftVersion(server.minecraft_version);
                setLoaderVersion(server.loader_version);
            }

            // Always refresh a file list when server changes
            loadAvailableFiles();
        } else
        {
            // Reset the ref when no server is selected
            firstLoadStateRef.current = false;
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
        <div className="flex flex-col gap-4 p-6 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] overflow-y-auto">
            <div className="flex items-center justify-between">
                <h2 className="text-xl font-minecraft-header">Server Configuration</h2>
                <Button
                    color="primary"
                    isLoading={isSaving || isUploadingLoader}
                    isDisabled={!hasChanges()}
                    onPress={handleSave}
                    startContent={<Icon icon="pixelarticons:save"/>}
                    className={"absolute right-10 -translate-y-[70px] z-10"}
                    // className={"data-[has-changes=true]:fixed data-[has-changes=false]:absolute data-[has-changes=false]:right-8 data-[has-changes=true]:right-16 z-10"}
                    data-has-changes={hasChanges()}
                >
                    {isUploadingLoader ? "Uploading Server..." : "Save Changes"}
                </Button>
            </div>

            {/* Basic Information */}
            <section className="flex flex-col gap-4">
                <h3 className="text-lg font-minecraft-header">Basic Information</h3>
                <div className={"flex flex-row gap-2"}>
                    <ServerIcon id={server.id} isChangeEnabled={true} size={"sm"}/>
                    <Input
                        label="Server Name"
                        radius="none"
                        className="font-minecraft-body"
                        value={name}
                        onValueChange={setName}
                        startContent={<Icon icon="pixelarticons:device-game-console"/>}
                    />
                </div>

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

            {/* Loader Configuration */}
            <section className="space-y-4">
                <h3 className="text-lg font-minecraft-header justify-between flex flex-row items-center">Server Type & Version
                    <Tooltip content={"Reinstall the selected server jar."}>
                        <Button isIconOnly onPress={installSelectedLoader}>
                            <Icon icon={"pixelarticons:repeat"}/>
                        </Button>
                    </Tooltip>
                </h3>

                <div className="mx-auto">
                    <Tabs
                        radius="none"
                        className="font-minecraft-body"
                        fullWidth
                        variant="solid"
                        color="primary"
                        classNames={{
                            tab: "flex flex-col items-center justify-center h-24 w-28"
                        }}
                        isDisabled={isSaving || isUploadingLoader}
                        selectedKey={loaderType.toLowerCase()}
                        onSelectionChange={async (key) =>
                        {
                            const newLoader = key as LoaderType;
                            setLoaderType(newLoader);
                            if (newLoader === "vanilla")
                            {
                                // Reset loader URL for vanilla
                                setLoaderUrl(await getMinecraftVersionDownloadUrl(minecraftVersion));
                                setLoaderVersion("");
                            }
                        }}
                    >
                        <Tab
                            key="vanilla"
                            title={
                                <>
                                    <Icon icon="heroicons:cube-transparent-16-solid" width={32}/>
                                    <p>Vanilla</p>
                                </>
                            }
                        />
                        <Tab
                            key="fabric"
                            title={
                                <div className="relative">
                                    <Icon icon="file-icons:fabric" width={32}/>
                                    <p>Fabric</p>
                                </div>
                            }
                        />
                        <Tab
                            key="forge"
                            title={
                                <>
                                    <Icon icon="simple-icons:curseforge" width={32}/>
                                    <p>Forge</p>
                                </>
                            }
                        />
                        <Tab
                            key="quilt"
                            title={
                                <div className="flex justify-center items-center flex-col gap-2">
                                    <Quilt size={32}/>
                                    <p>Quilt</p>
                                </div>
                            }
                        />
                        <Tab
                            key="neoforge"
                            title={
                                <div className="flex justify-center items-center flex-col gap-2">
                                    <NeoForge size={32}/>
                                    <p>NeoForge</p>
                                </div>
                            }
                        />
                        <Tab
                            key="custom"
                            title={
                                <div className="flex justify-center items-center flex-col gap-2">
                                    <Icon icon="pixelarticons:cloud-upload" width={32}/>
                                    <p>Custom</p>
                                </div>
                            }
                        />
                    </Tabs>
                </div>

                <MinecraftVersionSelector
                    onVersionChange={(version, url) =>
                    {
                        if (!version) return;
                        setMinecraftVersion(version);
                        // Store the vanilla server URL if this is for vanilla servers
                        if (loaderType === "vanilla" && url)
                        {
                            setLoaderUrl(url);
                        }
                    }}
                    version={minecraftVersion}
                    isDisabled={isSaving || isUploadingLoader}
                />

                <LoaderSelector
                    loaderVersion={loaderVersion}
                    selectedLoader={loaderType}
                    minecraftVersion={minecraftVersion}
                    isSnapshot={(minecraftVersion?.includes("snapshot") || minecraftVersion?.includes("pre-release")) ?? false}
                    onChange={handleLoaderChange}
                    onCustomJarChange={setCustomJarFile}
                    isDisabled={isSaving || isUploadingLoader}
                />

                {hasLoaderChanges() && (
                    <div className="bg-warning-50 border border-warning-200 rounded-lg p-4">
                        <div className="flex items-center gap-2 text-warning-700">
                            <Icon icon="pixelarticons:info-box"/>
                            <p className="font-minecraft-body text-sm">
                                Server type or version changes detected. A new server jar will be downloaded when you save.
                                {loaderType !== "custom" && loaderUrl && (
                                    <span className="block mt-1 opacity-75">
                                        New jar: {generateNewJarFilename()}
                                    </span>
                                )}
                            </p>
                        </div>
                    </div>
                )}
            </section>

            <Divider/>

            {/* Java Configuration */}
            <section className="flex flex-col gap-4">
                <h3 className="text-lg font-minecraft-header">Java Configuration</h3>

                <JavaExecutableSelector
                    defaultSelectedExecutable={javaExecutable}
                    onVersionChange={(executable) =>
                    {
                        if (executable) setJavaExecutable(executable);
                    }}
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

type LoaderSelectorProps = {
    selectedLoader: string;
    minecraftVersion: string | undefined;
    loaderVersion: string | undefined;
    onChange: (url: string | undefined, version: string | undefined) => void;
    onCustomJarChange: (file: File | undefined) => void;
    isDisabled: boolean;
    isSnapshot: boolean;
}

function LoaderSelector(props: LoaderSelectorProps)
{
    const {
        selectedLoader,
        minecraftVersion,
        loaderVersion,
        onChange,
        isDisabled
    } = props;

    if (!minecraftVersion)
    {
        return (
            <p className="text-danger font-minecraft-body text-tiny italic underline">
                Please select a Minecraft version first.
            </p>
        );
    }

    switch (selectedLoader.toLowerCase())
    {
        case "fabric":
            return (
                <FabricVersionSelector
                    version={loaderVersion}
                    minecraftVersion={minecraftVersion}
                    onVersionChange={onChange}
                    isDisabled={isDisabled}
                    isSnapshot={props.isSnapshot}
                />
            );
        case "forge":
            return (
                <ForgeVersionSelector
                    version={loaderVersion}
                    minecraftVersion={minecraftVersion}
                    onVersionChange={onChange}
                    isDisabled={isDisabled}
                />
            );
        case "quilt":
            return (
                <QuiltVersionSelector
                    version={loaderVersion}
                    minecraftVersion={minecraftVersion}
                    isDisabled={isDisabled}
                />
            );
        case "neoforge":
            return (
                <NeoForgeVersionSelector
                    version={loaderVersion}
                    minecraftVersion={minecraftVersion}
                    isDisabled={isDisabled}
                />
            );
        case "custom":
            return (
                <FileInput
                    accept=".jar,.zip,.tar.gz,.tar"
                    description="Upload your custom jar file or modpack archive."
                    multiple={false}
                    onChange={(file) => props.onCustomJarChange(file as File | undefined)}
                    readOnly={isDisabled}
                />
            );
        default:
            return null;
    }
}
