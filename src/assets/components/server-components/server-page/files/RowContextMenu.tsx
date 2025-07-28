import {Listbox, ListboxItem, ListboxSection} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {FilesystemEntry} from "../../../../ts/filesystem.ts";
import {useCallback, useEffect, useState} from "react";
import $ from "jquery";
import {isTextFile} from "../../../../ts/file-type-match.ts";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {MessageResponseType} from "../../../MessageModal.tsx";
import {useMessage} from "../../../../providers/MessageProvider.tsx";

export type ContextMenuOptions = {
    entry?: FilesystemEntry | FilesystemEntry[];
    x: number;
    y: number;
    isOpen: boolean;
}
type RowContextMenuProps = {
    onClose: () => void;
} & ContextMenuOptions;

export function RowContextMenu({entry, y, x, isOpen, onClose}: RowContextMenuProps)
{
    const {downloadEntry} = useServer();
    const {open} = useMessage();
    const [position, setPosition] = useState({x, y});


    const downloadSelectedEntries = useCallback(async () =>
    {
        if (!entry || !isOpen) return;
        onClose();
        await downloadEntry(entry);
    }, [entry, downloadEntry, isOpen]);

    const deleteSelectedEntries = useCallback(async () =>
    {
        if (!entry || !isOpen) return;

        onClose();
        let response = await open({
            title: "Delete Files",
            body: `Are you sure you want to delete ${Array.isArray(entry) ? `${entry.length} files` : entry.filename}? This action cannot be undone.`,
            responseType: MessageResponseType.OkayCancel,
            severity: "danger"
        });
        if (response)
        {
            // Implement delete logic here
        }
    }, [entry, isOpen]);

    useEffect(() =>
    {
        let parent = $("#server-file-browser");
        let menu = $("#server-files-context-menu");
        if (parent.length === 0 || menu.length == 0) return;

        let offset = parent.offset();
        let parentWidth = parent.width();
        let parentHeight = parent.height();
        let menuWidth = menu.outerWidth();
        let menuHeight = menu.outerHeight();
        if (!offset || !menuWidth || !menuHeight || !parentWidth || !parentHeight) return;


        let newX = x;
        let newY = y;
        if (newX + menuWidth > offset.left + parentWidth)
        {
            newX = (offset.left + parentWidth) - menuWidth - 10; // 10px padding
        }
        if (newY + menuHeight > offset.top + parentHeight)
        {
            newY = (offset.top + parentHeight) - menuHeight - 10; // 10px padding
        }

        setPosition({x: newX, y: newY});

    }, [x, y]);
    return (
        <Listbox
            id={"server-files-context-menu"}
            className={"absolute z-50 w-64 bg-background/50 backdrop-blur-sm border border-primary/50 shadow-lg data-[open=true]:opacity-100 data-[open=false]:opacity-0 transition-opacity duration-200 data-[open=false]:pointer-events-none font-minecraft-body"}
            style={{top: position.y, left: position.x}}
            itemClasses={{base: "rounded-none font-minecraft-body"}}
            data-open={isOpen}
            onSelectionChange={() => onClose()}
        >
            <ListboxSection title={Array.isArray(entry) ? `${entry.length} Items Selected` : entry?.filename ?? ""} itemClasses={{base: "rounded-none font-minecraft-body"}}>
                {!Array.isArray(entry) && entry ? (() =>
                {
                    let singleItemOptions = [];
                    if (!entry?.is_dir && isTextFile(entry?.path))
                    {
                        singleItemOptions.push(
                            <ListboxItem key={"edit"} endContent={<Icon icon={"pixelarticons:edit-box"}/>}>Edit</ListboxItem>
                        );
                    }


                    return (
                        <>
                            {...singleItemOptions}
                            <ListboxItem key={"rename"} endContent={<Icon icon={"pixelarticons:unlink"}/>}>Rename</ListboxItem>
                        </>
                    );
                })() : null}
                <ListboxItem key={"archive"} endContent={<Icon icon={"pixelarticons:archive"}/>}>Archive</ListboxItem>
                <ListboxItem key={"download"} endContent={<Icon icon={"pixelarticons:flatten"}/>} onPress={downloadSelectedEntries}>Download</ListboxItem>
                <ListboxItem key={"delete"} color={"danger"} className={"text-danger"} endContent={<Icon icon={"pixelarticons:trash"}/>} onPress={deleteSelectedEntries}>Delete</ListboxItem>
            </ListboxSection>
        </Listbox>
    );
}