import {Listbox, ListboxItem, ListboxSection} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {FilesystemEntry} from "../../../../ts/filesystem.ts";
import {useCallback, useEffect, useState} from "react";
import $ from "jquery";
import {isTextFile} from "../../../../ts/file-type-match.ts";
import {useServer} from "../../../../providers/ServerProvider.tsx";

export type ContextMenuOptions = {
    entry?: FilesystemEntry | FilesystemEntry[];
    x: number;
    y: number;
    isOpen: boolean;
}
type RowContextMenuProps = {
    onRename: (entry: FilesystemEntry) => void;
    onDelete: (entry: FilesystemEntry[]) => void;
    onArchive: (entry: FilesystemEntry[]) => void;
    onEdit: (entry: FilesystemEntry) => void;
    onClose: () => void;
} & ContextMenuOptions;

export function RowContextMenu({entry, y, x, isOpen, onClose, onRename, onDelete, onArchive, onEdit}: RowContextMenuProps)
{
    const {downloadEntry} = useServer();
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
        if (Array.isArray(entry)) onDelete(entry);
        else onDelete([entry]);
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

        setPosition({x: newX - 50, y: newY - 340});

        const handleClickOutside = (event: MouseEvent) =>
        {
            const clickTarget = event.target as HTMLElement;
            if (!menu[0].contains(clickTarget) && !parent[0].contains(clickTarget))
            {
                onClose();
            }
        };

        if (isOpen)
        {
            document.addEventListener("mousedown", handleClickOutside);
        }

        return () =>
        {
            document.removeEventListener("mousedown", handleClickOutside);
        };

    }, [x, y, isOpen, onClose]);
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
                            <ListboxItem key={"edit"} endContent={<Icon icon={"pixelarticons:edit-box"}/>} onPress={() => onEdit(entry)}>Edit</ListboxItem>
                        );
                    }


                    return (
                        <>
                            {...singleItemOptions}
                            <ListboxItem key={"rename"} endContent={<Icon icon={"pixelarticons:unlink"}/>} onPress={() =>
                            {
                                onRename(entry);
                                onClose();
                            }}>Rename</ListboxItem>
                        </>
                    );
                })() : null}
                <ListboxItem key={"archive"} endContent={<Icon icon={"pixelarticons:archive"}/>} onPress={() => onArchive(Array.isArray(entry) ? entry : [entry] as FilesystemEntry[])}>Archive</ListboxItem>
                <ListboxItem key={"download"} endContent={<Icon icon={"pixelarticons:flatten"}/>} onPress={downloadSelectedEntries}>Download</ListboxItem>
                <ListboxItem key={"delete"} color={"danger"} className={"text-danger"} endContent={<Icon icon={"pixelarticons:trash"}/>} onPress={deleteSelectedEntries}>Delete</ListboxItem>
            </ListboxSection>
        </Listbox>
    );
}