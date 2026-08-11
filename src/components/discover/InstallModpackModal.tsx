import {addToast, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useEffect, useState} from "react";
import {useNavigate} from "react-router-dom";
import JavaExecutableSelector from "../server-components/JavaExecutableSelector.tsx";
import {Button} from "../extended/Button.tsx";
import {installModpackServer} from "../../utils/modpack-api.ts";
import type {ModpackPlatform} from "../../types/ModpackTypes.ts";

type InstallModpackModalProps = {
    isOpen: boolean;
    onClose: () => void;
    platform: ModpackPlatform;
    packId: string;
    version: string;
    packName: string;
    minecraftVersion?: string;
};

export function InstallModpackModal(props: InstallModpackModalProps)
{
    const {isOpen, onClose, platform, packId, version, packName, minecraftVersion} = props;
    const navigate = useNavigate();
    const [name, setName] = useState(packName);
    const [javaExecutable, setJavaExecutable] = useState<string | undefined>(undefined);
    const [isInstalling, setIsInstalling] = useState(false);

    useEffect(() =>
    {
        if (isOpen) setName(packName);
    }, [isOpen, packName]);

    const install = async () =>
    {
        if (!javaExecutable || !name.trim()) return;
        setIsInstalling(true);
        try
        {
            const serverId = await installModpackServer({
                platform,
                packId,
                version,
                name: name.trim(),
                javaExecutable
            });
            addToast({
                title: "Installation Started",
                description: `Server "${name.trim()}" is being created. Mods and loader are downloading in the background.`,
                color: "success"
            });
            onClose();
            navigate(`/app/servers/${serverId}`);
        } catch (error: any)
        {
            addToast({
                title: "Installation Failed",
                description: error?.message || "Failed to start modpack installation.",
                color: "danger"
            });
        } finally
        {
            setIsInstalling(false);
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            backdrop={"blur"}
            radius={"none"}
            closeButton={<Icon icon={"pixelarticons:close-box"} width={24}/>}
            classNames={{closeButton: "rounded-none"}}
            isDismissable={!isInstalling}
            hideCloseButton={isInstalling}
        >
            <ModalContent>
                <ModalHeader className={"font-minecraft-header font-normal"}>Create Server from Modpack</ModalHeader>
                <ModalBody className={"flex flex-col gap-4 font-minecraft-body"}>
                    <p className={"text-default-500 text-sm"}>
                        Installing {packName} {version}{minecraftVersion ? ` (Minecraft ${minecraftVersion})` : ""}
                    </p>
                    <Input
                        label={"Server Name"}
                        radius={"none"}
                        size={"sm"}
                        value={name}
                        onValueChange={setName}
                        isDisabled={isInstalling}
                    />
                    <JavaExecutableSelector
                        minecraftVersion={minecraftVersion}
                        onVersionChange={setJavaExecutable}
                        isDisabled={isInstalling}
                    />
                </ModalBody>
                <ModalFooter className={"font-minecraft-body"}>
                    <Button
                        onPress={install}
                        variant={"ghost"}
                        color={"primary"}
                        isDisabled={!javaExecutable || !name.trim()}
                        isLoading={isInstalling}
                    >
                        Install
                    </Button>
                    <Button onPress={onClose} variant={"light"} color={"danger"} isDisabled={isInstalling}>
                        Cancel
                    </Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    );
}
