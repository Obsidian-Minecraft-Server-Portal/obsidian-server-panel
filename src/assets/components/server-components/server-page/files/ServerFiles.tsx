import {Button, ButtonGroup, Chip, cn, Input, Progress, Skeleton, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {KeyboardEvent, useCallback, useEffect, useState} from "react";
import {FilesystemData, FilesystemEntry} from "../../../../ts/filesystem.ts";
import "../../../../ts/math-ext.ts";
import {Icon} from "@iconify-icon/react";
import $ from "jquery";
import {ContextMenuOptions, RowContextMenu} from "./RowContextMenu.tsx";
import {useMessage} from "../../../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../../../MessageModal.tsx";
import {FileTableBreadcrumbs} from "./FileTableBreadcrumbs.tsx";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {ErrorBoundary} from "../../../ErrorBoundry.tsx";
import {FileEntryIcon} from "./FileEntryIcon.tsx";

type UploadProgress = {
    entry: FilesystemEntry;
    progress: number;
    files: string[]
    isUploading: boolean;
}


export function ServerFiles()
{
    const {getEntries, renameEntry, createEntry, deleteEntry, uploadFile, archiveFiles} = useServer();
    const {open} = useMessage();
    const [path, setPath] = useState("");
    const [data, setData] = useState<FilesystemData>();
    const [selectedEntries, setSelectedEntries] = useState<FilesystemEntry[]>([]);
    const [contextMenuOptions, setContextMenuOptions] = useState<ContextMenuOptions>({entry: undefined, x: 0, y: 0, isOpen: false});
    const [isLoading, setIsLoading] = useState(false);
    const [renamingEntry, setRenamingEntry] = useState<FilesystemEntry | undefined>(undefined);
    const [newItemCreationEntry, setNewItemCreationEntry] = useState<FilesystemEntry | undefined>(undefined);
    const [newArchiveEntry, setNewArchiveEntry] = useState<UploadProgress | undefined>(undefined);
    const [fileUploadEntries, setFileUploadEntries] = useState<UploadProgress[]>([]);
    const [isDraggingOver, setIsDraggingOver] = useState(false);

    const scrollToTop = useCallback(() =>
    {
        $("#server-files-table").parent().scrollTop(0);
    }, [path]);


    const upload = useCallback(async (files: File[]) =>
    {
        let promises = [];
        for (let file of files)
        {
            let entry = {filename: file.name, path, is_dir: false, size: file.size, file_type: file.type} as FilesystemEntry;
            setFileUploadEntries(prev => [...prev, {entry, progress: 0, files: [file.name], isUploading: true}]);
            promises.push(
                (async () =>
                {
                    await uploadFile(file, entry.path, progress =>
                        {
                            setFileUploadEntries(prev => prev.map(upload => upload.entry === entry ? {...upload, progress} : upload));
                            console.log("Upload progress:", progress);
                        }, async () =>
                        {
                            // On Canceled
                            setFileUploadEntries(prev => prev.filter(upload => upload.entry !== entry));
                            await refresh();
                        }
                    );
                    setFileUploadEntries(prev => prev.filter(upload => upload.entry !== entry));
                    await refresh();
                })()
            );
        }
        await Promise.all(promises);
        await refresh();
    }, [setFileUploadEntries]);


    const refresh = useCallback(async () =>
    {
        scrollToTop();
        setIsLoading(true);
        const data = await getEntries(path);
        data.entries = data.entries.sort((a, b) =>
        {
            if (a.is_dir && !b.is_dir) return -1; // Directories first
            if (!a.is_dir && b.is_dir) return 1; // Files after directories
            return a.filename.localeCompare(b.filename); // Sort alphabetically
        });
        setData(data);
        setIsLoading(false);
        setSelectedEntries([]);
        setContextMenuOptions({entry: undefined, x: 0, y: 0, isOpen: false});

    }, [path]);

    const renameSelectedEntry = useCallback(async (newName: string) =>
    {
        if (!renamingEntry || newName.trim() === "" || newName === renamingEntry.filename)
        {
            setRenamingEntry(undefined);
            return;
        }

        let newPath = `${path}/${newName}`;
        try
        {
            await renameEntry(renamingEntry.path, newPath);
            setRenamingEntry(undefined);
            await refresh();
        } catch (error)
        {
            console.error("Failed to rename entry:", error);
            await open({
                title: "Rename Failed",
                body: "An error occurred while renaming the entry. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    }, [renamingEntry, path]);

    const startEntryCreation = useCallback(async (directory: boolean) =>
    {
        scrollToTop();
        let filename = `New ${directory ? "Directory" : "File"}`;
        let index = 0;
        while (data?.entries.some(entry => entry.filename === filename))
        {
            index++;
            filename = `New ${directory ? "Directory" : "File"} (${index})`;
        }
        let entry = {filename, path, is_dir: directory, size: 0, file_type: directory ? "Directory" : "File"} as FilesystemEntry;
        setData(prev => ({...prev, entries: [entry, ...(prev?.entries || [])]} as FilesystemData));
        setNewItemCreationEntry(entry);
    }, [data, path]);

    const completeEntryCreation = useCallback(async (newName: string) =>
    {
        if (!newItemCreationEntry || newName.trim() === "" || newName === newItemCreationEntry.filename)
        {
            setNewItemCreationEntry(undefined);
            await refresh();
            return;
        }

        try
        {
            await createEntry(newName, path, newItemCreationEntry.is_dir);
            setNewItemCreationEntry(undefined);
            await refresh();
        } catch (error)
        {
            console.error("Failed to rename entry:", error);
            await open({
                title: "Creation Failed",
                body: "An error occurred while create new entry. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    }, [data, path]);

    const startArchiveCreation = useCallback(async () =>
    {
        setContextMenuOptions(prev => ({...prev, isOpen: false}));
        scrollToTop();
        let filename = "New Archive";
        let index = 0;
        while (data?.entries.some(entry => entry.filename === `${filename}.zip`))
        {
            index++;
            filename = `New Archive (${index})`;
        }
        let entry = {filename, path, is_dir: false, size: 0, file_type: "Archive"} as FilesystemEntry;
        setData(prev => ({...prev, entries: [entry, ...(prev?.entries || [])]} as FilesystemData));
        setNewArchiveEntry({entry, progress: 0, files: selectedEntries.map(entry => entry.path), isUploading: false});
    }, [path, data, selectedEntries]);
    const completeArchiveCreation = useCallback(async (newName: string) =>
    {
        setNewArchiveEntry(prev => prev ? {...prev, isUploading: true} : undefined);
        if (!newArchiveEntry || newName.trim() === "")
        {
            setNewArchiveEntry(undefined);
            await refresh();
            return;
        }

        try
        {
            archiveFiles(`${newName}.zip`, newArchiveEntry.files, path, progress =>
            {
                setNewArchiveEntry(prev => prev ? {...prev, progress} : undefined);
                console.log("Archive progress:", progress);
            }, async () =>
            {
                setNewArchiveEntry(undefined);
                await refresh();
            }, error =>
            {
                open({
                    title: "Archive Creation Failed",
                    body: `An error occurred while creating the archive: ${error}`,
                    responseType: MessageResponseType.Close,
                    severity: "danger"
                });
                console.error("Failed to create archive:", error);
            });
        } catch (error)
        {
            console.error("Failed to create archive:", error);
            await open({
                title: "Archive Creation Failed",
                body: "An error occurred while creating the archive. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    }, [data, path, newArchiveEntry]);

    const handleKeyDown = useCallback(async (e: KeyboardEvent<HTMLTableElement>) =>
    {

        if (renamingEntry !== undefined)
        {
            e.preventDefault();
            return;
        }

        if (e.key === "Escape")
        {
            setContextMenuOptions(prev => ({...prev, isOpen: false}));
        }
        if (e.key === "F2")
        {
            // Start renaming the first selected entry
            if (selectedEntries.length > 0)
            {
                e.preventDefault();
                setRenamingEntry(selectedEntries[0]);
                return;
            }
        }
        if (e.key === "Delete" || e.key === "Backspace")
        {
            await deleteSelected(selectedEntries);
        }
    }, [renamingEntry, refresh, renameSelectedEntry, selectedEntries]);

    const deleteSelected = useCallback(async (entries: FilesystemEntry[]) =>
    {
        // Handle delete action for selected entries
        if (entries.length > 0)
        {
            let response = await open({
                title: "Delete Files",
                body: `Are you sure you want to delete ${entries.length > 1 ? `${entries.length} files` : entries[0].filename}? This action cannot be undone.`,
                responseType: MessageResponseType.OkayCancel,
                severity: "danger"
            });
            if (response)
            {
                // Implement delete logic here
                await deleteEntry(entries.map(entry => entry.path));
                await refresh();
            }
        }
    }, []);

    useEffect(() =>
    {
        refresh().then();
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
        <div
            id={"server-file-browser"}
            className={
                cn(
                    "flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative"
                )
            }
            onDragStart={() => setIsDraggingOver(false)}
            onDragEnd={() => setIsDraggingOver(false)}
            onDragEnter={() => setIsDraggingOver(true)}
            onDragExit={() => setIsDraggingOver(false)}
            onDragOver={e => e.preventDefault()}
            onDrop={async e =>
            {
                e.preventDefault();
                console.log("Files dropped:", e.dataTransfer.files);
                await upload([...e.dataTransfer.files]);
                setIsDraggingOver(false);
            }}
            data-dragging-over={isDraggingOver}
        >
            {isDraggingOver && (
                <div className="absolute inset-0 z-30 border-dotted border-4 border-primary bg-background/90 flex items-center justify-center">
                    <span className="font-minecraft-body text-4xl">Drop Files to Upload</span>
                </div>
            )}

            <div className={"flex flex-row justify-between items-center"}>
                <FileTableBreadcrumbs onNavigate={setPath} paths={path.split("/").filter(p => p.trim() !== "")}/>
                <ButtonGroup radius={"none"} variant={"flat"}>
                    <Tooltip content={"New File"}>
                        <Button radius={"none"} isIconOnly className={"text-xl"} onPress={() => startEntryCreation(false)}>
                            <Icon icon={"pixelarticons:file-plus"}/>
                        </Button>
                    </Tooltip>
                    <Tooltip content={"New Directory"}>
                        <Button radius={"none"} isIconOnly className={"text-xl"} onPress={() => startEntryCreation(true)}>
                            <Icon icon={"pixelarticons:folder-plus"}/>
                        </Button>
                    </Tooltip>
                    <Tooltip content={"Refresh Files"}>
                        <Button radius={"none"} isIconOnly className={"text-xl"} isDisabled={isLoading} onPress={refresh}>
                            <Icon icon={"pixelarticons:repeat"}/>
                        </Button>
                    </Tooltip>
                </ButtonGroup>
            </div>
            <ErrorBoundary>
                <Table
                    id={"server-files-table"}
                    removeWrapper
                    radius={"none"}
                    className={cn("font-minecraft-body overflow-y-auto")}
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
                    isKeyboardNavigationDisabled={true}
                    onKeyDown={handleKeyDown}
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
                                    <>
                                        {fileUploadEntries.map(upload => (
                                            <TableRow key={upload.entry.filename}>
                                                <TableCell className={"flex items-center h-14 gap-2"}>
                                                    <FileEntryIcon entry={upload.entry}/> {upload.entry.filename}
                                                </TableCell>
                                                <TableCell className={"text-gray-500"}>{upload.entry.file_type}</TableCell>
                                                <TableCell className={"text-gray-500"}>
                                                    <Progress
                                                        minValue={0}
                                                        maxValue={100}
                                                        value={upload.progress}
                                                        size={"sm"}
                                                    />
                                                </TableCell>
                                                <TableCell>
                                                    <></>
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                        {data?.entries.map(entry =>

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
                                                    if (entry.is_dir && !renamingEntry && !newItemCreationEntry)
                                                    {
                                                        setPath(prev => prev ? `${prev}/${entry.filename}` : entry.filename);
                                                    }
                                                }}
                                            >
                                                <TableCell className={"flex items-center h-14 gap-2"}>
                                                    {renamingEntry === entry ?
                                                        <Input
                                                            startContent={<FileEntryIcon entry={entry}/>}
                                                            defaultValue={entry.filename}
                                                            autoFocus
                                                            onBlur={e => renameSelectedEntry(e.currentTarget.value)}
                                                            onKeyDown={async e =>
                                                            {
                                                                if (e.key === "Enter") await renameSelectedEntry(e.currentTarget.value);
                                                            }}
                                                            radius={"none"}
                                                            className={"font-minecraft-body"}
                                                        /> :
                                                        newItemCreationEntry === entry ?
                                                            <Input
                                                                startContent={<FileEntryIcon entry={entry}/>}
                                                                defaultValue={entry.filename}
                                                                autoFocus
                                                                onBlur={e => completeEntryCreation(e.currentTarget.value)}
                                                                onKeyDown={async e =>
                                                                {
                                                                    if (e.key === "Enter") await completeEntryCreation(e.currentTarget.value);
                                                                }}
                                                                radius={"none"}
                                                                className={"font-minecraft-body"}
                                                            />
                                                            : (newArchiveEntry?.entry === entry && !newArchiveEntry.isUploading) ?
                                                                <Input
                                                                    startContent={<FileEntryIcon entry={{filename: ".zip"} as FilesystemEntry}/>}
                                                                    defaultValue={entry.filename}
                                                                    autoFocus
                                                                    onBlur={e => completeArchiveCreation(e.currentTarget.value)}
                                                                    onKeyDown={async e =>
                                                                    {
                                                                        if (e.key === "Enter") await completeArchiveCreation(e.currentTarget.value);
                                                                    }}
                                                                    radius={"none"}
                                                                    className={"font-minecraft-body"}
                                                                    endContent={<Chip>.zip</Chip>}
                                                                />
                                                                :
                                                                <><FileEntryIcon entry={entry}/> {entry.filename}</>
                                                    }
                                                </TableCell>
                                                <TableCell className={"text-gray-500"}>{entry.file_type}</TableCell>
                                                <TableCell className={"text-gray-500"}>
                                                    {entry === newArchiveEntry?.entry ?
                                                        <>
                                                            <Progress
                                                                minValue={0}
                                                                maxValue={100}
                                                                value={newArchiveEntry.progress}
                                                                size={"sm"}
                                                            />
                                                        </>
                                                        :
                                                        <>
                                                            {entry.is_dir ? "-" : Math.convertToByteString(entry.size)}
                                                        </>
                                                    }
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
                                        )}
                                    </>
                                )}
                            </>
                        )}
                    </TableBody>
                </Table>
            </ErrorBoundary>
            <RowContextMenu
                {...contextMenuOptions}
                onRename={setRenamingEntry}
                onDelete={deleteSelected}
                onArchive={startArchiveCreation}
                onClose={() => setContextMenuOptions(prev => ({...prev, isOpen: false}))}
            />
        </div>
    );
}

