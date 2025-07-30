import {Button, ButtonGroup, Chip, cn, Input, Progress, Skeleton, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {KeyboardEvent, useCallback, useEffect, useRef, useState} from "react";
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
import {Editor} from "@monaco-editor/react";
import {getMonacoLanguage, isTextFile} from "../../../../ts/file-type-match.ts";
import {registerMinecraftPropertiesLanguage} from "../../../../ts/minecraft-properties-language.ts";
import {AnimatePresence, motion} from "framer-motion";

// Define the theme outside the component
const defineObsidianTheme = (monaco: any) =>
{
    monaco.editor.defineTheme("obsidian-editor-theme", {
        base: "vs-dark",
        inherit: true,
        rules: [
            {token: "key", foreground: "#47ebb4"},
            {token: "value", foreground: "#CE9178"},
            {token: "comment", foreground: "#57718e", fontStyle: "italic"},
            {token: "operator", foreground: "#0aa370"}
        ],
        colors: {
            "editor.background": "#0b0b0e"
        }
    });
};

type UploadProgress = {
    entry: FilesystemEntry;
    progress: number;
    files: string[]
    isUploading: boolean;
    uploadGroup?: string;
    filesProcessed?: number;
    totalFiles?: number;
    operationType: "upload" | "archive" | "extract";
}

export function ServerFiles()
{
    const {getEntries, renameEntry, createEntry, deleteEntry, uploadFile, archiveFiles, extractArchive, getFileContents, setFileContents} = useServer();
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
    const [isEditingFile, setIsEditingFile] = useState(false);
    const [selectedFileContents, setSelectedFileContents] = useState("");
    const [isDragging, setIsDragging] = useState(false);
    const [editorWidth, setEditorWidth] = useState(() =>
    {
        // Load saved width from localStorage or use default
        const savedWidth = localStorage.getItem("editor-width");
        return savedWidth ? parseInt(savedWidth, 10) : 400;
    });
    const editorRef = useRef<any>(null);
    const monacoRef = useRef<any>(null);
    const editorWrapperRef = useRef<HTMLDivElement>(null);
    const editorSaveTimerRef = useRef<number | undefined>(undefined);
    const [needsToSave, setNeedsToSave] = useState(false);
    const newContentRef = useRef<string>("");

    const selectedEntriesRef = useRef<FilesystemEntry[]>([]);

    useEffect(() =>
    {
        selectedEntriesRef.current = selectedEntries;
    }, [selectedEntries]);

    const scrollToTop = useCallback(() =>
    {
        $("#server-files-table").parent().scrollTop(0);
    }, [path]);

    const saveContent = useCallback(async () =>
    {
        const currentSelectedEntries = selectedEntriesRef.current;
        const file = currentSelectedEntries[0]?.path;

        console.log("Attempting to save content:", newContentRef, "Needs to save:", needsToSave, "File:", currentSelectedEntries);

        if (editorSaveTimerRef.current) clearTimeout(editorSaveTimerRef.current);
        setNeedsToSave(false);

        try
        {
            if (!file || !newContentRef.current)
            {
                console.warn("No file selected or content is empty, skipping save.");
                return;
            }

            // Save the content to the file
            console.log("Saving file:", file, newContentRef.current);
            await setFileContents(file, newContentRef.current);
            newContentRef.current = "";
        } catch (error)
        {
            console.error("Failed to save file:", error);
            await open({
                title: "Save File Failed",
                body: "An error occurred while saving the file. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    }, [newContentRef, editorSaveTimerRef, needsToSave, open]);

    const reboundSaveContent = useCallback(async () =>
    {
        const currentSelectedEntries = selectedEntriesRef.current;
        if (!currentSelectedEntries || currentSelectedEntries.length !== 1 || !isTextFile(currentSelectedEntries[0].path)) return;

        if (editorSaveTimerRef.current) clearTimeout(editorSaveTimerRef.current);
        editorSaveTimerRef.current = setTimeout(async () =>
        {
            await saveContent();
        }, 5000);
    }, [saveContent]);

    const upload = useCallback(async (files: File[]) =>
    {
        let uploadGroup = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
        let promises = [];
        for (let file of files)
        {
            let entry = {filename: file.name, path, is_dir: false, size: file.size, file_type: file.type, operationType: "upload"} as FilesystemEntry;
            setFileUploadEntries(prev => [...prev, {entry, progress: 0, files: [file.name], isUploading: true, uploadGroup, operationType: "upload"}]);
            let totalSize = file.size;
            const {promise} = await uploadFile(file, entry.path, async bytes =>
                {
                    let progress = bytes / totalSize;
                    setFileUploadEntries(prev => prev.map(upload => upload.entry === entry ? {...upload, progress} : upload));
                    console.log("Upload progress:", progress);
                }, async () =>
                {
                    // On Canceled
                    setFileUploadEntries(prev => prev.filter(upload => upload.entry !== entry));
                    await refresh();
                }
            );
            promises.push(promise);
        }
        await Promise.all(promises);
        await refresh();
        setFileUploadEntries(prev => prev.filter(upload => upload.uploadGroup !== uploadGroup));
    }, [setFileUploadEntries, fileUploadEntries, path]);

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
    }, [path, data]);

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
        let filename = `New ${directory ? "Directory" : "File.txt"}`;
        let index = 0;
        while (data?.entries.some(entry => entry.filename === filename))
        {
            index++;
            filename = `New ${directory ? "Directory" : "File"} (${index}).txt`;
        }
        let entry = {filename, path, is_dir: directory, size: 0, file_type: directory ? "Directory" : "File"} as FilesystemEntry;
        setData(prev => ({...prev, entries: [entry, ...(prev?.entries || [])]} as FilesystemData));
        setNewItemCreationEntry(entry);
    }, [data, path]);

    const completeEntryCreation = useCallback(async (newName: string) =>
    {
        if (!newItemCreationEntry || newName.trim() === "")
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
        setNewArchiveEntry({entry, progress: 0, files: selectedEntries.map(entry => entry.path), isUploading: false, operationType: "archive"});
    }, [path, data, selectedEntries]);

    const completeArchiveCreation = useCallback(async (newName: string) =>
    {
        setNewArchiveEntry(prev => prev ? {...prev, isUploading: true, operationType: "archive"} : undefined);
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
                setNewArchiveEntry(undefined);
            }, () =>
            {
                setNewArchiveEntry(undefined);
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
            setNewArchiveEntry(undefined);
        }
    }, [data, path, newArchiveEntry]);

    const handleExtract = useCallback(async (entry: FilesystemEntry, outputPath?: string) =>
    {
        scrollToTop();

        // Determine the output path - either provided or current directory
        const extractPath = outputPath || path;

        // Create a unique ID for this extraction operation
        const extractId = `extract-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;

        // Create a temporary entry to show progress
        const extractEntry: FilesystemEntry = {
            filename: `Extracting ${entry.filename}...`,
            path: `${path}/extracting-${entry.filename}`,
            is_dir: false,
            size: 0,
            file_type: "Extracting"
        };

        const progressEntry: UploadProgress = {
            entry: extractEntry,
            progress: 0,
            files: [entry.path],
            isUploading: true,
            operationType: "extract",
            filesProcessed: 0,
            totalFiles: 0,
            uploadGroup: extractId // Use the unique ID to track this specific extraction
        };

        setFileUploadEntries(prev => [...prev, progressEntry]);

        // Helper function to remove the progress entry
        const removeProgressEntry = () =>
        {
            setFileUploadEntries(prev => prev.filter(upload => upload.uploadGroup !== extractId));
        };

        try
        {
            const {trackerId} = extractArchive(
                entry.path,
                extractPath,
                (progress, filesProcessed, totalFiles) =>
                {
                    setFileUploadEntries(prev =>
                        prev.map(upload =>
                            upload.uploadGroup === extractId
                                ? {...upload, progress, filesProcessed, totalFiles}
                                : upload
                        )
                    );
                    console.log("Extract progress:", progress, "Files:", filesProcessed, "/", totalFiles);
                },
                async () =>
                {
                    // Success - remove progress entry and refresh
                    console.log("Extract completed successfully, removing progress entry");
                    removeProgressEntry();
                    await refresh();
                },
                (error) =>
                {
                    // Error - remove progress entry and show an error
                    console.error("Failed to extract archive:", error);
                    removeProgressEntry();
                    open({
                        title: "Extract Failed",
                        body: `An error occurred while extracting the archive: ${error}`,
                        responseType: MessageResponseType.Close,
                        severity: "danger"
                    });
                },
                () =>
                {
                    // Cancelled - remove progress entry
                    console.log("Extract cancelled, removing progress entry");
                    removeProgressEntry();
                }
            );

            // Store the cancel function and track ID for potential future use
            console.log("Extract operation started with track ID:", trackerId);

        } catch (error)
        {
            console.error("Failed to start extract:", error);
            removeProgressEntry();
            await open({
                title: "Extract Failed",
                body: "An error occurred while starting the extraction. Please try again.",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        }
    }, [path, extractArchive, open, refresh]);

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

        if (e.ctrlKey && e.key === "a")
        {
            // Select all entries
            e.preventDefault();
            setSelectedEntries(data?.entries || []);
            return;
        }

        if (e.key === "Delete" || e.key === "Backspace")
        {
            await deleteSelected(selectedEntries);
        }
    }, [renamingEntry, refresh, renameSelectedEntry, selectedEntries]);

    const deleteSelected = useCallback(async (entries: FilesystemEntry[]) =>
    {
        // Handle the delete action for selected entries
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
    }, [path]);

    useEffect(() =>
    {
        refresh().then();
    }, [path]);

    useEffect(() =>
    {
        $(document).on("click", e =>
        {
            // Close the context menu when clicking outside
            if (!$(e.target).closest("#server-files-context-menu").length)
            {
                setContextMenuOptions(prev => ({...prev, isOpen: false}));
            }
        }).on("blur", e =>
        {
            // Close the context menu when focus is lost
            if (!$(e.target).closest("#server-files-context-menu").length)
            {
                setContextMenuOptions(prev => ({...prev, isOpen: false}));
            }
        });
        $("#server-files-table").parent().on("scroll", () =>
        {
            // Close the context menu when scrolling
            setContextMenuOptions(prev => ({...prev, isOpen: false}));
        });
        return () =>
        {
            $(document).off("click");
            $(document).off("blur");
            $("#server-files-table").parent().off("scroll");
        };
    }, []);

    useEffect(() =>
    {
        if (selectedEntries.length === 1 && isTextFile(selectedEntries[0].path) && isEditingFile)
        {
            if (editorSaveTimerRef.current) clearTimeout(editorSaveTimerRef.current);
            setSelectedFileContents("");
            setNeedsToSave(false);

            // Load file contents for a single text file selection
            getFileContents(selectedEntries[0].path).then(async contents =>
            {
                setSelectedFileContents(contents);
                setIsEditingFile(true);
            }).catch(async error =>
            {
                console.error("Failed to load file contents:", error);
                await open({
                    title: "Load File Failed",
                    body: "An error occurred while loading the file contents. Please try again.",
                    responseType: MessageResponseType.Close,
                    severity: "danger"
                });
            });
        } else
        {
            // Reset file contents when selection changes or multiple files are selected
            setSelectedFileContents("");
            // setIsEditingFile(false);
        }
    }, [selectedEntries, isEditingFile]);

    const handleEditorMount = useCallback((editor: any, monaco: any) =>
    {
        editorRef.current = editor;
        monacoRef.current = monaco;

        // Define the theme first
        defineObsidianTheme(monaco);

        // Register the Minecraft properties language
        registerMinecraftPropertiesLanguage(monaco);

        // Set the theme after it's defined
        monaco.editor.setTheme("obsidian-editor-theme");

        // Override the toggle line comment keybinding
        editor.addCommand(
            monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyC,
            () =>
            {
                editor.trigger("keyboard", "editor.action.commentLine", {});
            }
        );

        // Optional: Disable the original Ctrl+/ keybinding
        editor.addCommand(
            monaco.KeyMod.CtrlCmd | monaco.KeyCode.Slash,
            () =>
            {
            }
        );

        editor.addCommand(
            monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF,
            () =>
            {
                editor.trigger("keyboard", "editor.action.formatDocument", {});
            }
        );

        editor.addCommand(
            monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS,
            async () =>
            {
                console.log("Saving content from editor");
                await saveContent();
            }
        );

        editor.addCommand(
            monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyD,
            async () =>
            {
                editor.trigger("keyboard", "editor.action.deleteLines", {});
            }
        );

        editor.addCommand(
            monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyD,
            async () =>
            {
                editor.trigger("keyboard", "editor.action.duplicateSelection", {});
            }
        );
    }, [selectedEntries, isEditingFile]);

    return (
        <div className={
            cn(
                "flex flex-row gap-2 bg-default-50 overflow-x-hidden border-2 border-default-500/10"
            )
        }>
            <div
                id={"server-file-browser"}
                className={
                    cn(
                        "flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative grow min-w-[300px]"
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
                    setIsDraggingOver(false);
                    await upload([...e.dataTransfer.files]);
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
                        <Tooltip content={"Toggle File Editor"}>
                            <Button radius={"none"} isIconOnly className={"text-xl"} onPress={() => setIsEditingFile(prev => !prev)} color={isEditingFile ? "primary" : "default"}>
                                <Icon icon={"pixelarticons:notes"}/>
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
                            <TableColumn hidden={isEditingFile}>Type</TableColumn>
                            <TableColumn hidden={isEditingFile}>Size</TableColumn>
                            <TableColumn width={48} hideHeader hidden={isEditingFile}>Action</TableColumn>
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
                                    {data?.entries?.length === 0 && fileUploadEntries.length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={4} className="text-center text-gray-500">
                                                This directory is empty
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        <>
                                            {fileUploadEntries.map(upload => (
                                                <TableRow key={`upload-${upload.entry.filename}`}>
                                                    <TableCell className={"flex items-center h-14 gap-2"}>
                                                        <FileEntryIcon entry={upload.entry}/> {upload.entry.filename}
                                                    </TableCell>
                                                    <TableCell className={"text-gray-500"}>{upload.entry.file_type}</TableCell>
                                                    <TableCell className={"text-gray-500"}>
                                                        <div className="flex flex-col gap-1">
                                                            <Progress
                                                                minValue={0}
                                                                maxValue={upload.operationType === "upload" ? 1 : 100}
                                                                value={upload.progress}
                                                                size={"sm"}
                                                            />
                                                            {upload.operationType === "extract" && upload.totalFiles && upload.totalFiles > 0 && (
                                                                <span className="text-xs text-gray-400">
                                                                    {upload.filesProcessed || 0}/{upload.totalFiles} files
                                                                </span>
                                                            )}
                                                        </div>
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
                                                    <TableCell className={"text-gray-500"} hidden={isEditingFile && selectedEntries.length === 1}>{entry.file_type}</TableCell>
                                                    <TableCell className={"text-gray-500"} hidden={isEditingFile && selectedEntries.length === 1}>
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
                                                    <TableCell className={"text-gray-500"} hidden={isEditingFile && selectedEntries.length === 1}>
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
                {(!isEditingFile || selectedEntries.length !== 1) ? (
                    <RowContextMenu
                        {...contextMenuOptions}
                        onRename={setRenamingEntry}
                        onDelete={deleteSelected}
                        onArchive={startArchiveCreation}
                        onExtract={handleExtract}
                        onEdit={() =>
                        {
                            setIsEditingFile(true);
                            setContextMenuOptions(prev => ({...prev, isOpen: false}));
                        }}
                        onClose={() => setContextMenuOptions(prev => ({...prev, isOpen: false}))}
                    />
                ) : null}
            </div>
            <motion.div
                id={"server-file-editor"}
                ref={editorWrapperRef}
                className={"max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative"}
                initial={{opacity: 0, width: 0}}
                animate={{
                    opacity: isEditingFile && selectedEntries.length === 1 ? 1 : 0,
                    width: isEditingFile && selectedEntries.length === 1 ? `${editorWidth}px` : "0"
                }}
                exit={{opacity: 0, width: 0}}
                transition={{duration: isDragging ? 0 : 0.3, ease: "easeInOut"}}
                data-editing-file={isEditingFile && selectedEntries.length === 1}
            >
                {isEditingFile && selectedEntries.length === 1 && isTextFile(selectedEntries[0].path) ? (
                    <Editor
                        className={"w-full h-full"}
                        theme={"obsidian-editor-theme"}
                        value={isEditingFile ? selectedFileContents : ""}
                        language={getMonacoLanguage(selectedEntries[0]?.path ?? "") ?? "auto"}
                        onMount={handleEditorMount}
                        width={`${editorWidth}px`}
                        onChange={async content =>
                        {
                            console.log("Editor content changed:", content);
                            newContentRef.current = content ?? "";
                            setNeedsToSave(true);
                            await reboundSaveContent();
                        }}
                        options={{
                            fontSize: 14,
                            minimap: {enabled: false},
                            lineNumbers: "on",
                            scrollBeyondLastLine: false,
                            automaticLayout: true,
                            wordWrap: "on",
                            tabSize: 2,
                            contextmenu: false,
                            autoClosingBrackets: "always",
                            autoClosingOvertype: "always",
                            autoClosingQuotes: "always",
                            quickSuggestions: {
                                other: true,
                                comments: false,
                                strings: true
                            },
                            suggestOnTriggerCharacters: true,
                            acceptSuggestionOnEnter: "on",
                            tabCompletion: "on",
                            wordBasedSuggestions: "matchingDocuments",
                            parameterHints: {
                                enabled: true,
                                cycle: true
                            },
                            formatOnPaste: true,
                            formatOnType: true,
                            matchBrackets: "always",
                            autoIndent: "full",
                            folding: true,
                            foldingStrategy: "indentation",
                            suggest: {
                                showKeywords: true,
                                showSnippets: true,
                                showFunctions: true,
                                showConstructors: true,
                                showFields: true,
                                showVariables: true,
                                showClasses: true,
                                showStructs: true,
                                showInterfaces: true,
                                showModules: true,
                                showProperties: true,
                                showEvents: true,
                                showOperators: true,
                                showUnits: true,
                                showValues: true,
                                showConstants: true,
                                showEnums: true,
                                showEnumMembers: true,
                                showColors: true,
                                showFiles: true,
                                showReferences: true,
                                showFolders: true,
                                showTypeParameters: true,
                                showUsers: true,
                                showIssues: true
                            }
                        }}
                    />
                ) : selectedEntries.length === 1 && !isTextFile(selectedEntries[0].path) ? (
                    <div className="flex items-center justify-center h-full">
                        <span className="text-gray-500 font-minecraft-body">Select a text file to edit</span>
                    </div>
                ) : null}

                {/* Resize Handle */}
                {isEditingFile && selectedEntries.length === 1 && (
                    <div
                        className={
                            cn(
                                "w-[8px] h-full bg-transparent transition-all duration-200 absolute left-0 top-0 cursor-ew-resize select-none hover:bg-primary hover:opacity-50"
                            )
                        }
                        data-dragging={isDragging}
                        onMouseDown={(e) =>
                        {
                            e.preventDefault();
                            setIsDragging(true);

                            const startX = e.clientX;
                            const startWidth = editorWidth;
                            const parentWidth = editorWrapperRef.current?.parentElement?.clientWidth;

                            const onMouseMove = (moveEvent: MouseEvent) =>
                            {
                                moveEvent.preventDefault();
                                const newWidth = startWidth - (moveEvent.clientX - startX);
                                if (!parentWidth) return;
                                setEditorWidth(Math.min(parentWidth - 300, Math.max(300, newWidth)));
                            };

                            const onMouseUp = (mouseEvent: MouseEvent) =>
                            {
                                mouseEvent.preventDefault();
                                setIsDragging(false);

                                const newWidth = Math.max(300, startWidth - (mouseEvent.clientX - startX));
                                localStorage.setItem("editor-width", newWidth.toString());
                                document.removeEventListener("mousemove", onMouseMove);
                                document.removeEventListener("mouseup", onMouseUp);
                            };

                            document.addEventListener("mousemove", onMouseMove);
                            document.addEventListener("mouseup", onMouseUp);
                        }}
                    >
                        <span
                            className={
                                cn(
                                    "w-px h-full bg-white opacity-20 transition-all duration-200 absolute left-0 top-0 cursor-ew-resize select-none",
                                    "hover:opacity-50 hover:bg-primary",
                                    "data-[dragging=true]:opacity-50 data-[dragging=true]:bg-primary"
                                )
                            }
                        />
                    </div>
                )}
            </motion.div>

            {isEditingFile && selectedEntries.length === 1 && isTextFile(selectedEntries[0].path) && (
                <div className={"absolute bottom-8 right-8 z-50"}>
                    <Tooltip content={"Save Content"}>
                        <Button radius={"none"} onPress={saveContent} isIconOnly isDisabled={!needsToSave} color={needsToSave ? "primary" : "default"} size={"lg"}>
                            <Icon icon={"pixelarticons:save"}/>
                        </Button>
                    </Tooltip>
                </div>
            )}

            {/* Overlay to prevent clicks during dragging */}
            <AnimatePresence>
                {isDragging && (
                    <motion.div
                        className="fixed inset-0 z-50 cursor-ew-resize select-none pointer-events-auto bg-primary/10"
                        initial={{opacity: 0}}
                        animate={{opacity: 1}}
                        exit={{opacity: 0}}
                        transition={{duration: 0.2}}
                        onClick={(e) => e.stopPropagation()}
                    />
                )}
            </AnimatePresence>
        </div>
    );
}
