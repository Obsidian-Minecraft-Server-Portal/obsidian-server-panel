import {ListBox, ListBoxItem, ListBoxSection} from "@heroui/react";
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
        <ListBox
            {...{
                id: "server-files-context-menu",
                ref: menuRef,
                className: "absolute z-50 w-64 bg-background/50 backdrop-blur-sm border border-primary/50 shadow-lg data-[open=true]:opacity-100 data-[open=false]:opacity-0 transition-opacity duration-200 data-[open=false]:pointer-events-none font-minecraft-body rounded-none",
                style: {top: position.y, left: position.x},
                "data-open": isOpen,
                onSelectionChange: () => onClose(),
            } as any}
        >
            <ListBoxSection aria-label={Array.isArray(entry) ? `${entry.length} Items Selected` : entry?.filename ?? ""}>
                {!Array.isArray(entry) && entry ? (() =>
                {
                    let singleItemOptions = [];
                    if (!entry?.is_dir && isTextFile(entry?.path))
                    {
                        singleItemOptions.push(
                            <ListBoxItem key={"edit"} onPress={() => onEdit(entry)}><span className="flex justify-between w-full">Edit <Icon icon={"pixelarticons:edit-box"}/></span></ListBoxItem>
                        );
                    }

                    // Add extract options for archive files
                    if (!entry?.is_dir && isArchiveFile(entry?.filename))
                    {
                        const archiveBaseName = getArchiveBaseName(entry.filename);
                        singleItemOptions.push(
                            <ListBoxItem key={"extract-here"} onPress={() => {
                                onExtract(entry);
                                onClose();
                            }}><span className="flex justify-between w-full">Extract Here <Icon icon={"pixelarticons:extract"}/></span></ListBoxItem>
                        );
                        singleItemOptions.push(
                            <ListBoxItem key={"extract-to-folder"} onPress={() => {
                                onExtract(entry, archiveBaseName);
                                onClose();
                            }}><span className="flex justify-between w-full">Extract to {archiveBaseName} <Icon icon={"pixelarticons:folder-open"}/></span></ListBoxItem>
                        );
                    }

                    return (
                        <>
                            {...singleItemOptions}
                            <ListBoxItem key={"rename"} onPress={() =>
                            {
                                onRename(entry);
                                onClose();
                            }}><span className="flex justify-between w-full">Rename <Icon icon={"pixelarticons:unlink"}/></span></ListBoxItem>
                        </>
                    );
                })() : null}
                <ListBoxItem key={"archive"} onPress={() => onArchive(Array.isArray(entry) ? entry : [entry] as FilesystemEntry[])}><span className="flex justify-between w-full">Archive <Icon icon={"pixelarticons:archive"}/></span></ListBoxItem>
                <ListBoxItem key={"download"} onPress={downloadSelectedEntries}><span className="flex justify-between w-full">Download <Icon icon={"pixelarticons:flatten"}/></span></ListBoxItem>
                <ListBoxItem key={"add-to-ignore"} onPress={() => {
                    onAddToIgnore(Array.isArray(entry) ? entry : [entry] as FilesystemEntry[]);
                    onClose();
                }}><span className="flex justify-between w-full">Add to .obakignore <Icon icon={"pixelarticons:eye-closed"}/></span></ListBoxItem>
                <ListBoxItem key={"delete"} className={"text-danger"} onPress={deleteSelectedEntries}><Icon icon={"pixelarticons:trash"}/> Delete</ListBoxItem>
            </ListBoxSection>
        </ListBox>
    );
}
