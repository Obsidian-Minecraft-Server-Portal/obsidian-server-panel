import {addToast, Button, CircularProgress, Input, Link, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {NeoForge} from "../icons/NeoForge.svg.tsx";
import Quilt from "../icons/Quilt.svg.tsx";
import {Tooltip} from "../extended/Tooltip.tsx";
import {useCallback, useEffect, useState} from "react";
import {MinecraftVersionSelector} from "./version-selectors/MinecraftVersionSelector.tsx";
import {ForgeVersionSelector} from "./version-selectors/ForgeVersionSelector.tsx";
import {FabricVersionSelector} from "./version-selectors/FabricVersionSelector.tsx";
import {FileInput} from "../extended/FileInput.tsx";
import {QuiltVersionSelector} from "./version-selectors/QuiltVersionSelector.tsx";
import {NeoForgeVersionSelector} from "./version-selectors/NeoForgeVersionSelector.tsx";
import {LoaderType, useServer} from "../../providers/ServerProvider.tsx";
import RamSlider from "./RamSlider.tsx";
import JavaExecutableSelector from "./JavaExecutableSelector.tsx";
import {getMinecraftVersionDownloadUrl} from "../../ts/minecraft-versions.ts";

type NewServerProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function NewServerModal(props: NewServerProperties)
{
    const {createServer, uploadFromUrl, uploadFile, updateServer} = useServer();
    const [ram, setRam] = useState(4); // Default RAM value
    const [selectedLoader, setSelectedLoader] = useState<LoaderType>("vanilla"); // Default selected loader
    const [loaderVersion, setLoaderVersion] = useState<string | undefined>(undefined);
    const [selectedMinecraftVersion, setSelectedMinecraftVersion] = useState<string | undefined>(undefined);
    const [selectedJavaExecutable, setSelectedJavaExecutable] = useState<string | undefined>(undefined);

    const [loaderUrl, setLoaderUrl] = useState<string | undefined>(undefined); // For custom loader URLs
    const [name, setName] = useState("");
    const [customJarFile, setCustomJarFile] = useState<File | undefined>(undefined); // For custom jar file uploads
    const [isCreatingServer, setIsCreatingServer] = useState(false);
    const [isValidForm, setIsValidForm] = useState(false);
    const [creationProgress, setCreationProgress] = useState(0);


    const submit = useCallback(async () =>
    {
        if (!selectedMinecraftVersion || !selectedLoader || !selectedJavaExecutable || !isValidForm)
        {
            console.log(`Invalid form submission: Minecraft Version: ${selectedMinecraftVersion}, Loader: ${selectedLoader}, Java Executable: ${selectedJavaExecutable}, Is Valid Form: ${isValidForm}`);
            return;
        }
        setIsCreatingServer(true);
        setCreationProgress(0.1); // Started creating server
        const filepath = selectedLoader === "vanilla" || selectedLoader === "custom" ? `server-${selectedMinecraftVersion}.jar` : `${selectedLoader}-${loaderVersion}-${selectedMinecraftVersion}-server.jar`;
        try
        {
            let serverId = await createServer({
                name,
                server_type: selectedLoader,
                minecraft_version: selectedMinecraftVersion,
                loader_version: loaderVersion ?? "",
                java_executable: selectedJavaExecutable
            });

            setCreationProgress(0.3); // Server created

            if (!serverId)
            {
                addToast({
                    title: "Error",
                    description: "Failed to create server. Please try again.",
                    color: "danger"
                });
                setIsCreatingServer(false);
                return;
            }

            if (selectedLoader !== "custom")
            {
                if (!loaderUrl && selectedLoader !== "vanilla")
                {
                    console.error("Loader URL is not defined for selected loader:", selectedLoader);
                    addToast({
                        title: "Error",
                        description: `Loader URL is not defined for selected loader: ${selectedLoader}. Please select a valid loader version.`,
                        color: "danger"
                    });
                    setIsCreatingServer(false);
                    return;
                }
                const onProgress = (progress: number, downloaded: number, total: number) =>
                {
                    setCreationProgress(0.3 + (progress / 100 * 0.5)); // Progress from 30% to 80%
                    console.log(`Downloading ${selectedLoader} server: ${progress}% (${downloaded}/${total} bytes)`);
                };
                const onSuccess = () =>
                {
                    setCreationProgress(0.8); // Download complete
                };
                const onError = (error: string) =>
                {
                    console.error("Error uploading server jar:", error);
                };
                try
                {
                    await uploadFromUrl(loaderUrl ?? await getMinecraftVersionDownloadUrl(selectedMinecraftVersion), filepath, onProgress, onSuccess, onError, serverId);
                    setCreationProgress(0.8); // Download complete after successful upload
                } catch (error)
                {
                    console.error("Error uploading server jar:", error);
                    addToast({
                        title: "Error",
                        description: "Failed to upload server jar. Please check the URL or try again.",
                        color: "danger"
                    });
                    setIsCreatingServer(false);
                    return;
                }
            } else
            {
                if (!customJarFile)
                {
                    addToast({
                        title: "Error",
                        description: "Please upload a custom jar file.",
                        color: "danger"
                    });
                    return;
                }
                const onProgress = (bytes: number) =>
                {
                    const percentage = bytes / customJarFile.size;
                    setCreationProgress(0.3 + (percentage * 0.5)); // Progress from 30% to 80%
                    console.log(`Uploading custom server jar: ${bytes} bytes uploaded of ${customJarFile.size} bytes ${Math.round(percentage * 100)}%`);
                };
                const onCancel = () =>
                {
                };
                try
                {
                    await uploadFile(customJarFile, filepath, onProgress, onCancel, serverId);
                    setCreationProgress(0.8); // Upload complete
                } catch (error)
                {
                    console.error("Error uploading custom jar file:", error);
                    addToast({
                        title: "Error",
                        description: "Failed to upload custom jar file. Please try again.",
                        color: "danger"
                    });
                    setIsCreatingServer(false);
                    return;
                }
            }

            try
            {
                await updateServer({max_memory: ram, server_jar: filepath}, serverId);
                setCreationProgress(1); // Server settings updated
            } catch (error)
            {
                console.error("Error updating server RAM:", error);
                addToast({
                    title: "Error",
                    description: "Failed to update server RAM. Please try again.",
                    color: "danger"
                });
                setIsCreatingServer(false);
                return;
            }

            setIsCreatingServer(false);
            props.onClose();
        } catch (error)
        {
            console.error("Error creating server:", error);

            addToast({
                title: "Error",
                description: "Failed to create server. Please try again.",
                color: "danger"
            });
        }
    }, [loaderUrl, selectedLoader, selectedMinecraftVersion, ram, selectedJavaExecutable, isValidForm, name]);

    useEffect(() =>
    {
        setIsValidForm(name.trim() !== "" && selectedMinecraftVersion !== undefined && (selectedLoader !== "custom" || loaderUrl !== undefined) && selectedJavaExecutable !== undefined);
    }, [loaderUrl, selectedLoader, selectedMinecraftVersion, name, selectedJavaExecutable]);

    return (
        <Modal
            isOpen={props.isOpen}
            onClose={props.onClose}
            backdrop={"blur"}
            radius={"none"}
            closeButton={<Icon icon={"pixelarticons:close-box"} width={24}/>}
            classNames={{closeButton: "rounded-none"}}
            size={"3xl"}
            scrollBehavior={"inside"}
            isDismissable={!isCreatingServer}
            hideCloseButton={isCreatingServer}
        >
            <ModalContent>
                {onClose => (
                    <>
                        <ModalHeader className={"font-minecraft-header"}>Create New Server</ModalHeader>
                        <ModalBody className={"flex flex-col gap-4"}>
                            <p className={"font-minecraft-body"}>Configure your server</p>
                            <Input
                                label={"Server Name"}
                                className={"font-minecraft-body"}
                                radius={"none"}
                                size={"sm"}
                                endContent={<Icon icon={""}/>}
                                value={name}
                                onValueChange={setName}
                                isDisabled={isCreatingServer}
                            />
                            <div className={"mx-auto"}>
                                <Tabs
                                    radius={"none"}
                                    className={"font-minecraft-body"}
                                    fullWidth
                                    variant={"solid"}
                                    color={"primary"}
                                    classNames={{
                                        tab: "flex flex-col items-center justify-center h-24 w-28"
                                    }}
                                    isDisabled={isCreatingServer}
                                    selectedKey={selectedLoader}
                                    onSelectionChange={key => setSelectedLoader(key as LoaderType)}
                                >
                                    <Tab key={"vanilla"} title={<><Icon icon={"heroicons:cube-transparent-16-solid"} width={32}/><p>Vanilla</p></>}/>
                                    <Tab key={"fabric"} title={<div className={"relative"}><Icon icon={"file-icons:fabric"} width={32}/><p>Fabric</p></div>}/>
                                    <Tab key={"forge"} title={<><Icon icon={"simple-icons:curseforge"} width={32}/><p>Forge</p></>}/>
                                    <Tab key={"quilt"} title={<div className={"flex justify-center items-center flex-col gap-2"}><Quilt size={32}/><p>Quilt</p></div>}/>
                                    <Tab key={"neo_forge"} title={<div className={"flex justify-center items-center flex-col gap-2"}><NeoForge size={32}/><p>NeoForge</p></div>}/>
                                    <Tab key={"custom"} title={<div className={"flex justify-center items-center flex-col gap-2"}><Icon icon={"pixelarticons:cloud-upload"} width={32}/><p>Custom Jar</p></div>}/>
                                </Tabs>
                            </div>
                            <div className={"flex flex-row gap-1 items-center text-gray-500 text-sm font-minecraft-body"}>
                                <Tooltip content={<p>For more information about these settings, <Link href={"https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel"}>visit the documentation</Link>.</p>}>
                                    <Icon icon={"pixelarticons:info-box"}/>
                                </Tooltip>
                                <p>You can change these settings later in your server options.</p>
                            </div>

                            <MinecraftVersionSelector onVersionChange={setSelectedMinecraftVersion} version={selectedMinecraftVersion} isDisabled={isCreatingServer}/>
                            <LoaderSelector
                                selectedLoader={selectedLoader}
                                version={selectedMinecraftVersion}
                                isSnapshot={(selectedMinecraftVersion?.includes("snapshot") || selectedMinecraftVersion?.includes("pre-release")) ?? false}
                                onChange={(url, version) =>
                                {
                                    setLoaderUrl(url);
                                    setLoaderVersion(version);
                                }}
                                onCustomJarChange={setCustomJarFile}
                                isDisabled={isCreatingServer}
                            />
                            <RamSlider value={ram} onValueChange={setRam} isDisabled={isCreatingServer}/>
                            <JavaExecutableSelector onVersionChange={setSelectedJavaExecutable} isDisabled={isCreatingServer}/>
                        </ModalBody>
                        <ModalFooter>
                            <Button onPress={submit} radius={"none"} variant={"ghost"} color={"primary"} isDisabled={!isValidForm || isCreatingServer}>
                                {isCreatingServer &&
                                    <CircularProgress
                                        minValue={0}
                                        maxValue={1}
                                        value={creationProgress}
                                        color={"primary"}
                                        size={"sm"}
                                        classNames={{
                                            svg: "h-6 w-6"
                                        }}
                                    />
                                }
                                Create
                            </Button>
                            <Button onPress={onClose} radius={"none"} variant={"light"} color={"danger"} isLoading={isCreatingServer}>Cancel</Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}

type LoaderSelectorProps = {
    selectedLoader: string;
    version: string | undefined;
    onChange: (url: string | undefined, version: string | undefined) => void;
    onCustomJarChange: (file: File | undefined) => void;
    isDisabled: boolean;
    isSnapshot: boolean;
}

function LoaderSelector(props: LoaderSelectorProps)
{
    const {
        selectedLoader,
        version,
        onChange,
        isDisabled
    } = props;
    if (!version) return <p className={"text-danger font-minecraft-body text-tiny italic underline"}>Please select a Minecraft version first.</p>;
    switch (selectedLoader)
    {
        case "fabric":
            return <FabricVersionSelector minecraftVersion={version} onVersionChange={onChange} isDisabled={isDisabled} isSnapshot={props.isSnapshot}/>;
        case "forge":
            return <ForgeVersionSelector minecraftVersion={version} onVersionChange={onChange} isDisabled={isDisabled}/>;
        case "quilt":
            return <QuiltVersionSelector minecraftVersion={version} isDisabled={isDisabled}/>;
        case "neo_forge":
            return <NeoForgeVersionSelector minecraftVersion={version} isDisabled={isDisabled}/>;
        case "custom":
            return (
                <FileInput
                    accept={".jar,.zip,.tar.gz,.tar"}
                    description={"Upload your custom jar file or modpack archive."}
                    multiple={false}
                    onChange={file => props.onCustomJarChange(file as File | undefined)}
                    readOnly={isDisabled}
                />
            );
        default:
            return null;
    }
}