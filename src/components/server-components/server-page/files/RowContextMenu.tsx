import {Listbox, ListboxItem, ListboxSection} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {FilesystemEntry} from "../../../../ts/filesystem.ts";
import {useCallback, useEffect, useRef, useState} from "react";
import $ from "jquery";
import {isTextFile} from "../../../../ts/file-type-match.ts";
import {useServer} from "../../../../providers/ServerProvider.tsx";

// Helper function to check if a file is an archive
const isArchiveFile = (filename: string): boolean => {
    const archiveExtensions = ['.zip', '.tar.gz', '.tgz', '.tar', '.rar', '.7z', '.gz'];
    const lowerFilename = filename.toLowerCase();
    return archiveExtensions.some(ext => lowerFilename.endsWith(ext));
};

// Helper function to get archive name without extension
const getArchiveBaseName = (filename: string): string => {
    if (filename.toLowerCase().endsWith('.tar.gz')) {
        return filename.slice(0, -7); // Remove .tar.gz
    } else if (filename.toLowerCase().endsWith('.tgz')) {
        return filename.slice(0, -4); // Remove .tgz
    } else {
        // Remove last extension for other formats
        const lastDotIndex = filename.lastIndexOf('.');
        return lastDotIndex > 0 ? filename.slice(0, lastDotIndex) : filename;
    }
};

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
    onExtract: (entry: FilesystemEntry, outputPath?: string) => void;
    onEdit: (entry: FilesystemEntry) => void;
    onAddToIgnore: (entry: FilesystemEntry[]) => void;
    onClose: () => void;
} & ContextMenuOptions;

export function RowContextMenu({entry, y, x, isOpen, onClose, onRename, onDelete, onArchive, onEdit, onExtract, onAddToIgnore}: RowContextMenuProps)
{
    const {downloadEntry} = useServer();
    const [position, setPosition] = useState({x, y});
    const menuRef = useRef<HTMLDivElement>(null);


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
        let menu = menuRef.current;
        if (parent.length === 0 || !menu) return;
        let menuElement = $(menu);

        let offset = parent.offset();
        let parentWidth = parent.width();
        let parentHeight = parent.height();
        let menuWidth = menuElement.outerWidth();
        let menuHeight = menuElement.outerHeight();
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
    }, [x, y, isOpen, onClose]);
    return (
        <Listbox
            id={"server-files-context-menu"}
            ref={menuRef}
            className={"absolute z-50 w-64 bg-background/50 backdrop-blur-sm border border-primary/50 shadow-lg data-[open=true]:opacity-100 data-[open=false]:opacity-0 transition-opacity duration-200 data-[open=false]:pointer-events-none font-minecraft-body"}
            style={{top: position.y, left: position.x}}
            itemClasses={{base: "rounded-none font-minecraft-body"}}
            data-open={isOpen}
            onSelectionChange={() => onClose()}
            tabIndex={1}
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

                    // Add extract options for archive files
                    if (!entry?.is_dir && isArchiveFile(entry?.filename))
                    {
                        const archiveBaseName = getArchiveBaseName(entry.filename);
                        singleItemOptions.push(
                            <ListboxItem key={"extract-here"} endContent={<Icon icon={"pixelarticons:extract"}/>} onPress={() => {
                                onExtract(entry);
                                onClose();
                            }}>Extract Here</ListboxItem>
                        );
                        singleItemOptions.push(
                            <ListboxItem key={"extract-to-folder"} endContent={<Icon icon={"pixelarticons:folder-open"}/>} onPress={() => {
                                onExtract(entry, archiveBaseName);
                                onClose();
                            }}>Extract to {archiveBaseName}</ListboxItem>
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
                <ListboxItem key={"add-to-ignore"} endContent={<Icon icon={"pixelarticons:eye-closed"}/>} onPress={() => {
                    onAddToIgnore(Array.isArray(entry) ? entry : [entry] as FilesystemEntry[]);
                    onClose();
                }}>Add to .obakignore</ListboxItem>
                <ListboxItem key={"delete"} color={"danger"} className={"text-danger"} endContent={<Icon icon={"pixelarticons:trash"}/>} onPress={deleteSelectedEntries}>Delete</ListboxItem>
            </ListboxSection>
        </Listbox>
    );
}
