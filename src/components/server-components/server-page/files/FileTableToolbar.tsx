import {Button, ButtonGroup} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../../extended/Tooltip.tsx";

interface FileTableToolbarProps {
    onCreateFile: () => void;
    onCreateDirectory: () => void;
    onUploadFolder: () => void;
    onToggleEditor: () => void;
    onRefresh: () => void;
    isEditingFile: boolean;
    isLoading: boolean;
}

export function FileTableToolbar({
    onCreateFile,
    onCreateDirectory,
    onUploadFolder,
    onToggleEditor,
    onRefresh,
    isEditingFile,
    isLoading
}: FileTableToolbarProps) {
    return (
        <ButtonGroup radius={"none"} variant={"flat"}>
            <Tooltip content={"New File"}>
                <Button radius={"none"} isIconOnly className={"text-xl"} onPress={onCreateFile}>
                    <Icon icon={"pixelarticons:file-plus"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"New Directory"}>
                <Button radius={"none"} isIconOnly className={"text-xl"} onPress={onCreateDirectory}>
                    <Icon icon={"pixelarticons:folder-plus"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Upload Folder"}>
                <Button
                    radius={"none"}
                    isIconOnly
                    className={"text-xl"}
                    onPress={onUploadFolder}
                >
                    <Icon icon={"pixelarticons:cloud-upload"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Toggle File Editor"}>
                <Button
                    radius={"none"}
                    isIconOnly
                    className={"text-xl"}
                    onPress={onToggleEditor}
                    color={isEditingFile ? "primary" : "default"}
                >
                    <Icon icon={"pixelarticons:notes"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Refresh Files"}>
                <Button
                    radius={"none"}
                    isIconOnly
                    className={"text-xl"}
                    isDisabled={isLoading}
                    onPress={onRefresh}
                >
                    <Icon icon={"pixelarticons:repeat"}/>
                </Button>
            </Tooltip>
        </ButtonGroup>
    );
}
