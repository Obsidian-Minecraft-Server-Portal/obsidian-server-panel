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
        <ButtonGroup>
            <Tooltip content={"New File"}>
                <Button isIconOnly className={"text-xl rounded-none rounded-none"} onPress={onCreateFile}>
                    <Icon icon={"pixelarticons:file-plus"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"New Directory"}>
                <Button isIconOnly className={"text-xl rounded-none"} onPress={onCreateDirectory}>
                    <Icon icon={"pixelarticons:folder-plus"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Upload Folder"}>
                <Button

                    isIconOnly
                    className={"text-xl rounded-none"}
                    onPress={onUploadFolder}
                >
                    <Icon icon={"pixelarticons:cloud-upload"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Toggle File Editor"}>
                <Button

                    isIconOnly
                    className={"text-xl rounded-none"}
                    onPress={onToggleEditor}
                    variant={isEditingFile ? "primary" : "secondary"}
                >
                    <Icon icon={"pixelarticons:notes"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Refresh Files"}>
                <Button

                    isIconOnly
                    className={"text-xl rounded-none"}
                    isDisabled={isLoading}
                    onPress={onRefresh}
                >
                    <Icon icon={"pixelarticons:repeat"}/>
                </Button>
            </Tooltip>
        </ButtonGroup>
    );
}
