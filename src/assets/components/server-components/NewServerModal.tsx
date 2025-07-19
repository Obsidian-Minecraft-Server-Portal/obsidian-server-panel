import {Button, Input, Link, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Slider, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {NeoForge} from "../icons/NeoForge.svg.tsx";
import Quilt from "../icons/Quilt.svg.tsx";
import {Tooltip} from "../extended/Tooltip.tsx";
import {useState} from "react";
import {MinecraftVersionSelector} from "./version-selectors/MinecraftVersionSelector.tsx";
import {ForgeVersionSelector} from "./version-selectors/ForgeVersionSelector.tsx";
import {FabricVersionSelector} from "./version-selectors/FabricVersionSelector.tsx";
import {FileInput} from "../extended/FileInput.tsx";
import {QuiltVersionSelector} from "./version-selectors/QuiltVersionSelector.tsx";
import {NeoForgeVersionSelector} from "./version-selectors/NeoForgeVersionSelector.tsx";

type NewServerProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function NewServerModal(props: NewServerProperties)
{
    const [selectedLoader, setSelectedLoader] = useState("vanilla"); // Default selected loader
    const [selectedMinecraftVersion, setSelectedMinecraftVersion] = useState<string | undefined>(undefined);
    const [ram, setRam] = useState(4); // Default RAM value
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
                                    onSelectionChange={key => setSelectedLoader(key as string)}
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
                            <LoaderSelector selectedLoader={selectedLoader} version={selectedMinecraftVersion}/>

                            <Slider
                                minValue={2}
                                maxValue={64}
                                defaultValue={4}
                                step={1}
                                label={"Configured RAM (GB)"}
                                className={"font-minecraft-body text-nowrap"}
                                showTooltip
                                value={ram}
                                onChange={value => setRam(value as number)}
                                tooltipValueFormatOptions={{}}
                                marks={[
                                    {
                                        value: 2,
                                        label: "2 GB"
                                    },
                                    {
                                        value: 64,
                                        label: "64 GB"
                                    }
                                ]}

                                renderValue={() =>
                                    <Input
                                        radius={"none"}
                                        className={"w-16"}
                                        size={"sm"}
                                        value={ram.toString()}
                                        onChange={e => setRam(+e.target.value)}
                                        maxLength={3}
                                        type={"number"}
                                    />
                                }
                            />
                        </ModalBody>
                        <ModalFooter>
                            <Button onPress={onClose} radius={"none"} variant={"ghost"} color={"primary"}>Create</Button>
                            <Button onPress={onClose} radius={"none"} variant={"light"} color={"danger"}>Cancel</Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}


function LoaderSelector({selectedLoader, version}: { selectedLoader: string, version: string | undefined })
{
    if (!version) return <p className={"text-danger font-minecraft-body text-tiny italic underline"}>Please select a Minecraft version first.</p>;
    switch (selectedLoader)
    {
        case "fabric":
            return <FabricVersionSelector minecraftVersion={version}/>;
        case "forge":
            return <ForgeVersionSelector minecraftVersion={version}/>;
        case "quilt":
            return <QuiltVersionSelector minecraftVersion={version}/>;
        case "neo_forge":
            return <NeoForgeVersionSelector minecraftVersion={version}/>;
        case "custom":
            return (
                <FileInput
                    accept={".jar,.zip,.tar.gz,.tar"}
                    description={"Upload your custom jar file or modpack archive."}
                />
            );
        default:
            return null;
    }
}