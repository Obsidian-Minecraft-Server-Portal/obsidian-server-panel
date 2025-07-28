import {Button, Skeleton, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {useEffect, useState} from "react";
import {FilesystemData, FilesystemEntry} from "../../../../ts/filesystem.ts";
import {FileEntryIcon} from "./FileEntryIcon.tsx";
import "../../../../ts/math-ext.ts";
import {Icon} from "@iconify-icon/react";
import $ from "jquery";
import {ContextMenuOptions, RowContextMenu} from "./RowContextMenu.tsx";
import {useMessage} from "../../../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../../../MessageModal.tsx";
import {FileTableBreadcrumbs} from "./FileTableBreadcrumbs.tsx";


export function ServerFiles()
{
    const {getEntries} = useServer();
    const {open} = useMessage();
    const [path, setPath] = useState("");
    const [data, setData] = useState<FilesystemData>();
    const [selectedEntries, setSelectedEntries] = useState<FilesystemEntry[]>([]);
    const [contextMenuOptions, setContextMenuOptions] = useState<ContextMenuOptions>({entry: undefined, x: 0, y: 0, isOpen: false});
    const [isLoading, setIsLoading] = useState(false);

    useEffect(() =>
    {
        setIsLoading(true);
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
            .then(setData)
            .finally(() => setIsLoading(false));
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
        }).on("blur", e =>
        {
            // Close context menu when focus is lost
            if (!$(e.target).closest("#server-files-context-menu").length)
            {
                setContextMenuOptions(prev => ({...prev, isOpen: false}));
            }
        });
        $("#server-files-table").parent().on("scroll", () =>
        {
            // Close context menu when scrolling
            setContextMenuOptions(prev => ({...prev, isOpen: false}));
        });
        return () =>
        {
            $(document).off("click");
            $(document).off("blur");
            $("#server-files-table").parent().off("scroll");
        };
    }, []);

    return (
        <div id={"server-file-browser"} className={"flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px]"}>
            <FileTableBreadcrumbs onNavigate={setPath} paths={path.split("/").filter(p => p.trim() !== "")}/>
            <Table
                id={"server-files-table"}
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
                onKeyDown={async e =>
                {
                    e.preventDefault();
                    console.log("hi");
                    if (e.key === "Escape")
                    {
                        setContextMenuOptions(prev => ({...prev, isOpen: false}));
                    }
                    if (e.key === "Delete" || e.key === "Backspace")
                    {
                        e.preventDefault();
                        // Handle delete action for selected entries
                        if (selectedEntries.length > 0)
                        {
                            let response = await open({
                                title: "Delete Files",
                                body: `Are you sure you want to delete ${selectedEntries.length > 1 ? `${selectedEntries.length} files` : selectedEntries[0].filename}? This action cannot be undone.`,
                                responseType: MessageResponseType.OkayCancel,
                                severity: "danger"
                            });
                            if (response)
                            {
                                // Implement delete logic here
                                setSelectedEntries([]);
                            }
                        }
                    }
                }}
            >
                <TableHeader>
                    <TableColumn>Name</TableColumn>
                    <TableColumn>Type</TableColumn>
                    <TableColumn>Size</TableColumn>
                    <TableColumn width={48} hideHeader>Action</TableColumn>
                </TableHeader>
                <TableBody>
                    {isLoading ? Array.from({length: 5}, (_, i) => (
                        <TableRow key={`skeleton-${i}`}>
                            <TableCell className={"flex items-center h-14 gap-2"}>
                                <Skeleton className={"w-8 h-8"}/>
                                <Skeleton className={"w-32 h-6"}/>
                            </TableCell>
                            <TableCell>
                                <Skeleton className={"w-24 h-6"}/>
                            </TableCell>
                            <TableCell>
                                <Skeleton className={"w-16 h-6"}/>
                            </TableCell>
                            <TableCell>
                                <Skeleton className={"w-8 h-6"}/>
                            </TableCell>
                        </TableRow>
                    )) : (
                        <>
                            {data?.entries?.length === 0 ? (
                                <TableRow>
                                    <TableCell colSpan={4} className="text-center text-gray-500">
                                        This directory is empty
                                    </TableCell>
                                </TableRow>
                            ) : (
                                data?.entries.map(entry => (
                                    <TableRow
                                        key={entry.filename}
                                        onContextMenu={e =>
                                        {
                                            e.preventDefault();
                                            setContextMenuOptions({
                                                entry: selectedEntries.length > 1 ? selectedEntries : entry,
                                                x: e.clientX - 30,
                                                y: e.clientY - 50,
                                                isOpen: true
                                            });
                                        }}
                                        data-selected={contextMenuOptions.entry === entry && contextMenuOptions.isOpen}
                                        className={"data-[selected=true]:opacity-50 data-[selected=true]:bg-white/10"}
                                        onDoubleClick={() =>
                                        {
                                            if (entry.is_dir)
                                            {
                                                setPath(prev => prev ? `${prev}/${entry.filename}` : entry.filename);
                                            }
                                        }}
                                    >
                                        <TableCell className={"flex items-center h-14 gap-2"}>
                                            <FileEntryIcon entry={entry}/> {entry.filename}
                                        </TableCell>
                                        <TableCell className={"text-gray-500"}>{entry.file_type}</TableCell>
                                        <TableCell className={"text-gray-500"}>
                                            {entry.is_dir ? "-" : Math.convertToByteString(entry.size)}
                                        </TableCell>
                                        <TableCell className={"text-gray-500"}>
                                            <Button
                                                isIconOnly
                                                radius={"none"}
                                                variant={"light"}
                                                onPress={e =>
                                                {
                                                    let position = $(e.target).offset();
                                                    if (!position) return;
                                                    setContextMenuOptions({
                                                        entry,
                                                        x: position.left - 264,
                                                        y: position.top,
                                                        isOpen: true
                                                    });
                                                }}
                                            >
                                                <Icon icon={"pixelarticons:more-horizontal"}/>
                                            </Button>
                                        </TableCell>
                                    </TableRow>
                                ))
                            )}
                        </>
                    )}
                </TableBody>
            </Table>
            <RowContextMenu {...contextMenuOptions} onClose={() => setContextMenuOptions(prev => ({...prev, isOpen: false}))}/>
        </div>
    );
}

