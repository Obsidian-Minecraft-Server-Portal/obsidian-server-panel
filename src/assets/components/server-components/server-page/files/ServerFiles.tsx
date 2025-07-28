import {Button, Listbox, ListboxItem, ListboxSection, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useEffect, useState} from "react";
import {FilesystemData, FilesystemEntry} from "../../../../ts/filesystem.ts";
import {FileEntryIcon} from "./FileEntryIcon.tsx";
import "../../../../ts/math-ext.ts";
import {Icon} from "@iconify-icon/react";
import $ from "jquery";

type ContextMenuOptions = {
    entry?: FilesystemEntry | FilesystemEntry[];
    x: number;
    y: number;
    isOpen: boolean;
}

export function ServerFiles()
{
    const {getEntries} = useServer();
    const [path, setPath] = useState("");
    const [data, setData] = useState<FilesystemData>();
    const [selectedEntries, setSelectedEntries] = useState<FilesystemEntry[]>([]);
    const [contextMenuOptions, setContextMenuOptions] = useState<ContextMenuOptions>({entry: undefined, x: 0, y: 0, isOpen: false});

    useEffect(() =>
    {
        getEntries(path)
            .then(data =>
            {
                data.entries = data.entries.sort((a, b) =>
                {
                    if (a.is_dir && !b.is_dir) return -1; // Directories first
                    if (!a.is_dir && b.is_dir) return 1; // Files after directories
                    return a.filename.localeCompare(b.filename); // Sort alphabetically
                });
                return data;
            })
            .then(setData);
    }, [path]);

    useEffect(() =>
    {
        $(document).on("click", e =>
        {
            // Close context menu when clicking outside
            if (!$(e.target).closest("#server-files-context-menu").length)
            {
                setContextMenuOptions(prev => ({...prev, isOpen: false}));
            }
        });
        $(document).on("blur", e =>
        {
            // Close context menu when focus is lost
            if (!$(e.target).closest("#server-files-context-menu").length)
            {
                setContextMenuOptions(prev => ({...prev, isOpen: false}));
            }
        });
    }, []);

    return (
        <div className={"flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px]"}>
            <Table
                removeWrapper
                radius={"none"}
                className={"font-minecraft-body overflow-y-auto"}
                fullWidth
                color={"primary"}
                aria-label={"Server Files"}
                selectionMode={"multiple"}
                selectionBehavior={"replace"}
                showSelectionCheckboxes={false}
                isHeaderSticky
                classNames={{
                    tr: "!rounded-none",
                    th: "backdrop-blur-md bg-default-50/50 !rounded-none"
                }}
                selectedKeys={selectedEntries.map(entry => entry.filename)}
                onSelectionChange={keys =>
                {
                    setContextMenuOptions(prev => ({...prev, isOpen: false}));
                    const selected = [...keys].map(key => data?.entries.find(entry => entry.filename === key)).filter(Boolean) as FilesystemEntry[];
                    setSelectedEntries(selected);
                }}
            >
                <TableHeader>
                    <TableColumn>Name</TableColumn>
                    <TableColumn>Type</TableColumn>
                    <TableColumn>Size</TableColumn>
                    <TableColumn width={48} hideHeader>Action</TableColumn>
                </TableHeader>
                <TableBody>
                    {data?.entries.map(entry => (
                        <TableRow
                            key={entry.filename}
                            onContextMenu={e =>
                            {
                                e.preventDefault();
                                setContextMenuOptions({entry: selectedEntries.length > 1 ? selectedEntries : entry, x: e.clientX - 30, y: e.clientY - 50, isOpen: true});
                            }}
                            data-selected={contextMenuOptions.entry === entry && contextMenuOptions.isOpen}
                            className={"data-[selected=true]:opacity-50 data-[selected=true]:bg-white/10"}
                        >
                            <TableCell className={"flex items-center h-14 gap-2"}><FileEntryIcon entry={entry}/> {entry.filename}</TableCell>
                            <TableCell className={"text-gray-500"}>{entry.file_type}</TableCell>
                            <TableCell className={"text-gray-500"}>{entry.is_dir ? "-" : Math.convertToByteString(entry.size)}</TableCell>
                            <TableCell className={"text-gray-500"}>
                                <Button isIconOnly radius={"none"} variant={"light"} onPress={e =>
                                {
                                    setContextMenuOptions({entry, x: e.x - 30, y: e.y - 50, isOpen: true});
                                }}>
                                    <Icon icon={"pixelarticons:more-horizontal"}/>
                                </Button>
                            </TableCell>
                        </TableRow>
                    )) || (<></>)}
                </TableBody>
            </Table>
            <RowContextMenu {...contextMenuOptions} onClose={() => setContextMenuOptions(prev => ({...prev, isOpen: false}))}/>
        </div>
    );
}

type RowContextMenuProps = {
    onClose: () => void;
} & ContextMenuOptions;

function RowContextMenu({entry, y, x, isOpen, onClose}: RowContextMenuProps)
{
    return (
        <Listbox
            id={"server-files-context-menu"}
            className={"absolute z-50 w-64 bg-background/50 backdrop-blur-sm border border-primary/50 shadow-lg data-[open=true]:opacity-100 data-[open=false]:opacity-0 transition-opacity duration-200 data-[open=false]:pointer-events-none"}
            style={{top: y, left: x}}
            itemClasses={{base: "rounded-none font-minecraft-body"}}
            data-open={isOpen}
            onSelectionChange={() => onClose()}
        >
            <ListboxSection title={Array.isArray(entry) ? `${entry.length} Items Selected` : entry?.filename ?? ""} itemClasses={{base: "rounded-none font-minecraft-body"}}>
                <ListboxItem key={"archive"} endContent={<Icon icon={"pixelarticons:flatten"}/>}>Archive</ListboxItem>
                <ListboxItem key={"download"} endContent={<Icon icon={"pixelarticons:flatten"}/>}>Download</ListboxItem>
                <ListboxItem key={"delete"} color={"danger"} className={"text-danger"} endContent={<Icon icon={"pixelarticons:trash"}/>}>Delete</ListboxItem>
            </ListboxSection>
        </Listbox>
    );
}