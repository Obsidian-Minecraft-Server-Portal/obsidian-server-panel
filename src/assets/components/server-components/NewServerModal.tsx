import {addToast, Button, Input, Link, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {NeoForge} from "../icons/NeoForge.svg.tsx";
import Quilt from "../icons/Quilt.svg.tsx";
import {Tooltip} from "../extended/Tooltip.tsx";
import {useCallback, useState} from "react";
import {MinecraftVersionSelector} from "./version-selectors/MinecraftVersionSelector.tsx";
import {ForgeVersionSelector} from "./version-selectors/ForgeVersionSelector.tsx";
import {FabricVersionSelector} from "./version-selectors/FabricVersionSelector.tsx";
import {FileInput} from "../extended/FileInput.tsx";
import {QuiltVersionSelector} from "./version-selectors/QuiltVersionSelector.tsx";
import {NeoForgeVersionSelector} from "./version-selectors/NeoForgeVersionSelector.tsx";
import {LoaderType, useServer} from "../../providers/ServerProvider.tsx";
import RamSlider from "./RamSlider.tsx";

type NewServerProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function NewServerModal(props: NewServerProperties)
{
    const {createServer, uploadFromUrl, uploadFile} = useServer();
    const [ram, setRam] = useState(4); // Default RAM value
    const [selectedLoader, setSelectedLoader] = useState<LoaderType>("vanilla"); // Default selected loader
    const [loaderVersion, setLoaderVersion] = useState<string | undefined>(undefined);
    const [selectedMinecraftVersion, setSelectedMinecraftVersion] = useState<string | undefined>(undefined);
    const [selectedJavaExecutable, setSelectedJavaExecutable] = useState<string | undefined>(undefined);

    const [loaderUrl, setLoaderUrl] = useState<string | undefined>(undefined); // For custom loader URLs
    const [name, setName] = useState("");
    const [customJarFile, setCustomJarFile] = useState<File | undefined>(undefined); // For custom jar file uploads


    const submit = useCallback(async () =>
    {
        if (!selectedMinecraftVersion)
        {
            return;
        }
        try
        {
            let serverId = await createServer({
                name: "New Server",
                server_type: selectedLoader,
                minecraft_version: selectedMinecraftVersion,
                loader_version: loaderVersion ?? ""
            });

            if (!serverId)
            {
                addToast({
                    title: "Error",
                    description: "Failed to create server. Please try again.",
                    color: "danger"
                });
            }

            if (selectedLoader !== "custom")
            {
                if (!loaderUrl)
                {
                    console.error("Loader URL is not defined for selected loader:", selectedLoader);
                    addToast({
                        title: "Error",
                        description: `Loader URL is not defined for selected loader: ${selectedLoader}. Please select a valid loader version.`,
                        color: "danger"
                    });
                    return;
                }
                const filepath = `${selectedLoader}-${loaderVersion}-${selectedMinecraftVersion}-server.jar`;
                const onProgress = (progress: number, downloaded: number, total: number) =>
                {
                    console.log(`Downloading ${selectedLoader} server: ${progress}% (${downloaded}/${total} bytes)`);
                };
                const onSuccess = () =>
                {
                };
                const onError = (error: string) =>
                {
                    console.error("Error uploading server jar:", error);
                };
                await uploadFromUrl(loaderUrl, filepath, onProgress, onSuccess, onError, serverId);
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
                const filepath = `${name}-${selectedMinecraftVersion}-server.jar`;
                const onProgress = (bytes: number) =>
                {
                    console.log(`Uploading custom server jar: ${bytes} bytes uploaded of ${customJarFile.size} bytes ${Math.round((bytes / customJarFile.size) * 100)}%`);
                };
                const onCancel = () =>
                {
                };
                await uploadFile(customJarFile, filepath, onProgress, onCancel, serverId);
            }


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
    }, [loaderUrl, selectedLoader, selectedMinecraftVersion, ram]);

    const isValidForm = useCallback(() =>
    {
        return name.trim() !== "" && selectedMinecraftVersion !== undefined && (selectedLoader !== "custom" || loaderUrl !== undefined) && selectedJavaExecutable !== undefined;
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

                            <MinecraftVersionSelector onVersionChange={setSelectedMinecraftVersion} version={selectedMinecraftVersion}/>
                            <LoaderSelector
                                selectedLoader={selectedLoader}
                                version={selectedMinecraftVersion}
                                onChange={(url, version) =>
                                {
                                    setLoaderUrl(url);
                                    setLoaderVersion(version);
                                }}
                                onCustomJarChange={setCustomJarFile}
                            />
                            <RamSlider value={ram} onValueChange={setRam}/>
                        </ModalBody>
                        <ModalFooter>
                            <Button onPress={submit} radius={"none"} variant={"ghost"} color={"primary"} isDisabled={!isValidForm()}>Create</Button>
                            <Button onPress={onClose} radius={"none"} variant={"light"} color={"danger"}>Cancel</Button>
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
}

function LoaderSelector(props: LoaderSelectorProps)
{
    const {
        selectedLoader,
        version,
        onChange
    } = props;
    if (!version) return <p className={"text-danger font-minecraft-body text-tiny italic underline"}>Please select a Minecraft version first.</p>;
    switch (selectedLoader)
    {
        case "fabric":
            return <FabricVersionSelector minecraftVersion={version} onVersionChange={onChange}/>;
        case "forge":
            return <ForgeVersionSelector minecraftVersion={version} onVersionChange={onChange}/>;
        case "quilt":
            return <QuiltVersionSelector minecraftVersion={version}/>;
        case "neo_forge":
            return <NeoForgeVersionSelector minecraftVersion={version}/>;
        case "custom":
            return (
                <FileInput
                    accept={".jar,.zip,.tar.gz,.tar"}
                    description={"Upload your custom jar file or modpack archive."}
                    multiple={false}
                    onChange={file => props.onCustomJarChange(file as File | undefined)}
                />
            );
        default:
            return null;
    }
}