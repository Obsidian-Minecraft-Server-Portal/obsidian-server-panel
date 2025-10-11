import {Button, cn, Input, Skeleton, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import React, {KeyboardEvent, useCallback, useEffect, useRef, useState} from "react";
import {FilesystemData, FilesystemEntry} from "../../../../ts/filesystem.ts";
import "../../../../ts/math-ext.ts";
import $ from "jquery";
import {ContextMenuOptions, RowContextMenu} from "./RowContextMenu.tsx";
import {useMessage} from "../../../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../../../MessageModal.tsx";
import {FileTableBreadcrumbs} from "./FileTableBreadcrumbs.tsx";
import {ErrorBoundary} from "../../../ErrorBoundry.tsx";
import {isTextFile} from "../../../../ts/file-type-match.ts";
import {ServerFileEditor, ServerFileEditorRef} from "./ServerFileEditor.tsx";
import {AnimatePresence, motion} from "framer-motion";
import {FileTableToolbar} from "./FileTableToolbar.tsx";
import {FileEntryIcon} from "./FileEntryIcon.tsx";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {Icon} from "@iconify-icon/react";

// Add a tiny helper to get dirname from a relative path like "a/b/c.txt" -> "a/b"
const dirname = (p: string) =>
{
    const i = p.lastIndexOf("/");
    return i === -1 ? "" : p.slice(0, i);
};

// Shape we use internally to carry a file with its relative path inside a selected folder
type FileWithRelPath = { file: File; relativePath: string };

export function ServerFiles()
{
    const {getEntries, renameEntry, createEntry, deleteEntry, uploadFile, archiveFiles, extractArchive, getFileContents, setFileContents, downloadEntry} = useServer();
    const {open} = useMessage();
    const [path, setPath] = useState("");
    const [data, setData] = useState<FilesystemData>();
    const [selectedEntries, setSelectedEntries] = useState<FilesystemEntry[]>([]);
    const [contextMenuOptions, setContextMenuOptions] = useState<ContextMenuOptions>({entry: undefined, x: 0, y: 0, isOpen: false});
    const [isLoading, setIsLoading] = useState(false);
    const [renamingEntry, setRenamingEntry] = useState<FilesystemEntry | undefined>(undefined);
    const [newItemCreationEntry, setNewItemCreationEntry] = useState<FilesystemEntry | undefined>(undefined);
    const [isDraggingOver, setIsDraggingOver] = useState(false);
    const [isEditingFile, setIsEditingFile] = useState(localStorage.getItem("is-editing-file") === "true");
    const [selectedFileContents, setSelectedFileContents] = useState("");
    const [isDragging, setIsDragging] = useState(false);
    const [browserWidth, setBrowserWidth] = useState(() =>
    {
        // Load saved width from localStorage or use default
        const savedWidth = localStorage.getItem("browser-width");
        return savedWidth ? parseInt(savedWidth, 10) : 500;
    });
    const [needsToSave, setNeedsToSave] = useState(false);
    const [isExternallyModified, setIsExternallyModified] = useState(false);
    const newContentRef = useRef<string>("");
    const originalContentHashRef = useRef<string>("");
    const serverFileEditorRef = useRef<ServerFileEditorRef>(null);
    const folderInputRef = useRef<HTMLInputElement>(null); // + add a hidden <input> for folder selection

    const selectedEntriesRef = useRef<FilesystemEntry[]>([]);
    const containerRef = useRef<HTMLDivElement>(null);
    const loadedFilePathRef = useRef<string>(""); // Track which file is currently loaded

    // Simple hash function to detect content changes
    const hashString = useCallback((str: string) =>
    {
        let hash = 0;
        for (let i = 0; i < str.length; i++)
        {
            const char = str.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32-bit integer
        }
        return hash.toString();
    }, []);

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

    useEffect(() =>
    {
        selectedEntriesRef.current = selectedEntries;
    }, [selectedEntries]);

    useEffect(() =>
    {
        localStorage.setItem("is-editing-file", isEditingFile.toString());
    }, [isEditingFile]);

    const scrollToTop = useCallback(() =>
    {
        $("#server-files-table").parent().scrollTop(0);
    }, [path]);

    const saveContent = useCallback(async () =>
    {
        const currentSelectedEntries = selectedEntriesRef.current;
        const file = currentSelectedEntries[0]?.path;

        console.log("Attempting to save content:", newContentRef, "Needs to save:", needsToSave, "File:", currentSelectedEntries);

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
    }, [newContentRef, needsToSave, open, setFileContents]);

    const handleContentChange = useCallback((content: string) =>
    {
        console.log("Editor content changed:", content);
        newContentRef.current = content;
        setNeedsToSave(true);
    }, []);

    const handleDragStart = useCallback(() =>
    {
        setIsDragging(true);
    }, []);

    const handleDragEnd = useCallback(() =>
    {
        setIsDragging(false);
    }, []);

    const handleWidthChange = useCallback((width: number) =>
    {
        setBrowserWidth(width);
        console.log("Browser width changed:", width);
    }, []);

    const checkUnsavedChanges = useCallback(async () =>
    {
        if (needsToSave)
        {
            const response = await open({
                title: "Unsaved Changes",
                body: "You have unsaved changes. Do you want to discard them?",
                responseType: MessageResponseType.OkayCancel,
                severity: "warning"
            });

            if (response)
            {
                // User confirmed, discard changes
                setNeedsToSave(false);
                newContentRef.current = "";
                return true;
            }

            // User cancelled
            return false;
        }

        // No unsaved changes
        return true;
    }, [needsToSave, open, newContentRef]);

    const checkForExternalModifications = useCallback(async () =>
    {
        if (selectedEntries.length === 1 && isTextFile(selectedEntries[0].path) && isEditingFile && !needsToSave)
        {
            try
            {
                const currentContent = await getFileContents(selectedEntries[0].path);
                const currentHash = hashString(currentContent);

                if (originalContentHashRef.current && currentHash !== originalContentHashRef.current)
                {
                    setIsExternallyModified(true);
                }
            } catch (error)
            {
                console.error("Failed to check for external modifications:", error);
            }
        }
    }, [selectedEntries, isEditingFile, needsToSave, getFileContents, hashString]);

    const handleRefreshFileContents = useCallback(async () =>
    {
        if (selectedEntries.length === 1 && isTextFile(selectedEntries[0].path))
        {
            // Check for unsaved changes before refreshing
            if (needsToSave)
            {
                const canProceed = await checkUnsavedChanges();
                if (!canProceed)
                {
                    return;
                }
            }

            try
            {
                const contents = await getFileContents(selectedEntries[0].path);
                setSelectedFileContents(contents);
                setNeedsToSave(false);
                setIsExternallyModified(false);
                newContentRef.current = "";
                originalContentHashRef.current = hashString(contents);
                loadedFilePathRef.current = selectedEntries[0].path; // Update loaded file reference
            } catch (error)
            {
                console.error("Failed to refresh file contents:", error);
                await open({
                    title: "Refresh Failed",
                    body: "An error occurred while refreshing the file contents. Please try again.",
                    responseType: MessageResponseType.Close,
                    severity: "danger"
                });
            }
        }
    }, [selectedEntries, needsToSave, getFileContents, checkUnsavedChanges, open, hashString]);

    const handleSelectionChange = useCallback(async (keys: any) =>
    {
        const selected = [...keys].map(key => data?.entries.find(entry => entry.filename === key)).filter(Boolean) as FilesystemEntry[];

        // Check if we're navigating away from a file with unsaved changes
        if (needsToSave && selectedEntries.length > 0 && selected.length > 0)
        {
            // Check if the selection actually changed
            const selectionChanged = selectedEntries.length !== selected.length ||
                selectedEntries[0]?.path !== selected[0]?.path;

            if (selectionChanged)
            {
                const canProceed = await checkUnsavedChanges();
                if (!canProceed)
                {
                    // User cancelled, keep current selection
                    return;
                }
            }
        }

        setContextMenuOptions(prev => ({...prev, isOpen: false}));
        setSelectedEntries(selected);
    }, [needsToSave, selectedEntries, data, checkUnsavedChanges]);

    const handleToggleEditor = useCallback(async () =>
    {
        // If closing the editor with unsaved changes, confirm first
        if (isEditingFile && needsToSave)
        {
            const canProceed = await checkUnsavedChanges();
            if (!canProceed)
            {
                return;
            }
        }

        setIsEditingFile(prev => !prev);
    }, [isEditingFile, needsToSave, checkUnsavedChanges]);

    const handleNavigate = useCallback(async (newPath: string) =>
    {
        // Check for unsaved changes before navigating to a different directory
        if (needsToSave)
        {
            const canProceed = await checkUnsavedChanges();
            if (!canProceed)
            {
                return;
            }
        }

        setPath(newPath);
        loadedFilePathRef.current = ""; // Clear loaded file when navigating
    }, [needsToSave, checkUnsavedChanges]);

    // NEW: upload of files with their relative folder path
    const uploadWithRelPaths = useCallback(async (items: FileWithRelPath[]) =>
    {
        if (items.length === 0) return;

        const promises: Promise<void>[] = [];

        for (const {file, relativePath} of items)
        {
            // Use the relative path (excluding the filename) to preserve folders
            const relDir = dirname(relativePath);
            const targetDir = relDir ? (path ? `${path}/${relDir}` : relDir) : path;

            const {promise} = await uploadFile(file, targetDir);
            promises.push(promise);
        }

        await Promise.all(promises);
        await refresh();
    }, [path, uploadFile, refresh]);

    // Existing single-level upload still available for plain files
    const upload = useCallback(async (files: File[]) =>
    {
        // Try to detect relative paths (webkitRelativePath). If present, route to uploadWithRelPaths instead.
        const items: FileWithRelPath[] = files.map(f =>
        {
            const rel = (f as any).webkitRelativePath || "";
            return {file: f, relativePath: rel};
        });
        const hasRel = items.some(i => i.relativePath && i.relativePath.length > 0);
        if (hasRel)
        {
            return uploadWithRelPaths(items);
        }

        const promises: Promise<void>[] = [];
        for (const file of files)
        {
            const {promise} = await uploadFile(file, path);
            promises.push(promise);
        }
        await Promise.all(promises);
        await refresh();
    }, [path, uploadFile, refresh, uploadWithRelPaths]);

    // Helper: recursively collect files from a DataTransferItem (drag-and-drop folder)
    const collectFromEntry = useCallback(async (entry: any, prefix: string): Promise<FileWithRelPath[]> =>
    {
        if (!entry) return [];

        // File
        if (entry.isFile)
        {
            const file: File = await new Promise((resolve) => entry.file(resolve));
            // record relative path as prefix + filename
            const relativePath = prefix ? `${prefix}/${file.name}` : file.name;
            return [{file, relativePath}];
        }

        // Directory
        if (entry.isDirectory)
        {
            const dirReader = entry.createReader();
            const entries: any[] = await new Promise((resolve) =>
            {
                const all: any[] = [];
                const readBatch = () =>
                {
                    dirReader.readEntries((batch: any[]) =>
                    {
                        if (batch.length)
                        {
                            all.push(...batch);
                            readBatch();
                        } else
                        {
                            resolve(all);
                        }
                    });
                };
                readBatch();
            });

            let results: FileWithRelPath[] = [];
            for (const child of entries)
            {
                const nextPrefix = prefix ? `${prefix}/${entry.name}` : entry.name;
                const childFiles = await collectFromEntry(child, nextPrefix);
                results = results.concat(childFiles);
            }
            return results;
        }

        return [];
    }, []);

    // Collect files from a drop event (supports folders)
    const collectDroppedFiles = useCallback(async (e: React.DragEvent): Promise<FileWithRelPath[]> =>
    {
        const dt = e.dataTransfer;
        if (dt.items && dt.items.length > 0)
        {
            const tasks: Promise<FileWithRelPath[]>[] = [];
            for (const item of Array.from(dt.items))
            {
                const entry = (item as any).webkitGetAsEntry ? (item as any).webkitGetAsEntry() : null;
                if (entry)
                {
                    tasks.push(collectFromEntry(entry, ""));
                } else if (item.kind === "file")
                {
                    const file = item.getAsFile();
                    if (file) tasks.push(Promise.resolve([{file, relativePath: file.name}]));
                }
            }
            const batches = await Promise.all(tasks);
            return batches.flat();
        }

        // Fallback to plain files (no directories)
        return Array.from(dt.files).map((f) => ({file: f, relativePath: f.name}));
    }, [collectFromEntry]);


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
    }, [renamingEntry, path, renameEntry, refresh, open]);

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
    }, [data, path, scrollToTop]);

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
    }, [newItemCreationEntry, path, createEntry, refresh, open]);

    const startArchiveCreation = useCallback(async () =>
    {
        setContextMenuOptions(prev => ({...prev, isOpen: false}));

        let filename = "New Archive";
        let index = 0;
        while (data?.entries.some(entry => entry.filename === `${filename}.zip`))
        {
            index++;
            filename = `New Archive (${index})`;
        }

        // Prompt user for archive name
        const archiveName = prompt("Enter archive name:", filename);
        if (!archiveName || archiveName.trim() === "") return;

        try
        {
            archiveFiles(
                `${archiveName}.zip`,
                selectedEntries.map(entry => entry.path),
                path,
                () =>
                {
                }, // on_progress - no longer needed
                async () => await refresh(), // on_success
                (error) =>
                {
                    open({
                        title: "Archive Creation Failed",
                        body: `An error occurred while creating the archive: ${error}`,
                        responseType: MessageResponseType.Close,
                        severity: "danger"
                    });
                }
            );
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
    }, [path, data, selectedEntries, archiveFiles, refresh, open]);


    const handleExtract = useCallback(async (entry: FilesystemEntry, outputPath?: string) =>
    {
        // Determine the output path - either provided or current directory
        const extractPath = outputPath || path;

        try
        {
            extractArchive(
                entry.path,
                extractPath,
                () =>
                {
                }, // on_progress - no longer needed
                async () => await refresh(), // on_success
                (error) =>
                {
                    // Error - show an error
                    console.error("Failed to extract archive:", error);
                    open({
                        title: "Extract Failed",
                        body: `An error occurred while extracting the archive: ${error}`,
                        responseType: MessageResponseType.Close,
                        severity: "danger"
                    });
                }
            );
        } catch (error)
        {
            console.error("Failed to start extract:", error);
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
            const selectedFilePath = selectedEntries[0].path;

            // Load file contents if:
            // 1. No file is currently loaded, OR
            // 2. A different file is selected (file switched), OR
            // 3. Content is empty and there are no unsaved changes
            const shouldLoadFile =
                loadedFilePathRef.current === "" ||
                loadedFilePathRef.current !== selectedFilePath ||
                (selectedFileContents === "" && !needsToSave);

            if (shouldLoadFile)
            {
                // Load file contents for a single text file selection
                getFileContents(selectedFilePath).then(async contents =>
                {
                    setSelectedFileContents(contents);
                    setIsEditingFile(true);
                    setIsExternallyModified(false);
                    setNeedsToSave(false);
                    newContentRef.current = "";
                    originalContentHashRef.current = hashString(contents);
                    loadedFilePathRef.current = selectedFilePath; // Track the loaded file
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
            }
        } else
        {
            // Reset file contents when selection changes or multiple files are selected
            setSelectedFileContents("");
            originalContentHashRef.current = "";
            setIsExternallyModified(false);
            loadedFilePathRef.current = ""; // Clear the loaded file reference
        }
    }, [selectedEntries, isEditingFile, getFileContents, open, hashString, needsToSave, selectedFileContents]);

    // Periodically check for external modifications
    useEffect(() =>
    {
        if (!isEditingFile || selectedEntries.length !== 1) return;

        const interval = setInterval(() =>
        {
            checkForExternalModifications();
        }, 3000); // Check every 3 seconds

        return () => clearInterval(interval);
    }, [isEditingFile, selectedEntries, checkForExternalModifications]);

    return (
        <div
            ref={containerRef}
            className={
                cn(
                    "flex flex-col gap-2 bg-default-50 border-2 border-default-500/10"
                )
            }
        >
            {/* Hidden folder input for "Choose Folder" */}
            <input
                ref={folderInputRef}
                type="file"
                multiple
                // @ts-expect-error: non-standard but widely supported
                webkitdirectory="true"
                style={{display: "none"}}
                onChange={async (e) =>
                {
                    const files = Array.from(e.currentTarget.files || []);
                    // Each File may include webkitRelativePath
                    await upload(files);
                    e.currentTarget.value = ""; // reset
                }}
            />

            {/* Breadcrumbs and Toolbar - Always full width */}
            <div className={"flex flex-row justify-between items-center px-4 pt-4"}>
                <FileTableBreadcrumbs onNavigate={handleNavigate} paths={path.split("/").filter(p => p.trim() !== "")}/>
                <FileTableToolbar
                    onCreateFile={() => startEntryCreation(false)}
                    onCreateDirectory={() => startEntryCreation(true)}
                    onUploadFolder={() => folderInputRef.current?.click()}
                    onToggleEditor={handleToggleEditor}
                    onRefresh={refresh}
                    isEditingFile={isEditingFile}
                    isLoading={isLoading}
                />
            </div>

            {/* Content area with file browser and editor */}
            <div className={"flex flex-row gap-2 overflow-x-hidden"}>
                <div
                    id={"server-file-browser"}
                    className={
                        cn(
                            "flex flex-col gap-2 px-4 pb-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative min-w-[300px]"
                        )
                    }
                    style={{
                        width: isEditingFile && selectedEntries.length === 1 ? `${browserWidth}px` : "100%",
                        transition: isDragging ? "none" : "width 0.3s ease-in-out"
                    }}
                    onDragStart={() => setIsDraggingOver(false)}
                    onDragEnd={() => setIsDraggingOver(false)}
                    onDragEnter={() => setIsDraggingOver(true)}
                    onDragExit={() => setIsDraggingOver(false)}
                    onDragOver={e => e.preventDefault()}
                    onDrop={async e =>
                    {
                        e.preventDefault();
                        setIsDraggingOver(false);
                        // Collect recursively if folders are dropped
                        const items = await collectDroppedFiles(e);
                        await uploadWithRelPaths(items);
                    }}
                    data-dragging-over={isDraggingOver}
                >
                    {isDraggingOver && (
                        <div className="absolute inset-0 z-30 border-dotted border-4 border-primary bg-background/90 flex items-center justify-center">
                            <span className="font-minecraft-body text-4xl">Drop Files or Folders to Upload</span>
                        </div>
                    )}
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
                        onSelectionChange={handleSelectionChange}
                        isKeyboardNavigationDisabled={true}
                        onKeyDown={handleKeyDown}
                    >
                        <TableHeader>
                            <TableColumn>Name</TableColumn>
                            <TableColumn hidden={isEditingFile && selectedEntries.length === 1}>Type</TableColumn>
                            <TableColumn hidden={isEditingFile && selectedEntries.length === 1}>Size</TableColumn>
                            <TableColumn width={48} hideHeader hidden={isEditingFile && selectedEntries.length === 1}>Action</TableColumn>
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
                                            {data?.entries.map(entry =>
                                            {
                                                const isSelected = selectedEntries.length === 1 && selectedEntries[0] === entry;
                                                const isRenaming = renamingEntry === entry;
                                                const isNewItem = newItemCreationEntry === entry;

                                                return (
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
                                                        onDoubleClick={async () =>
                                                        {
                                                            if (renamingEntry || newItemCreationEntry) return;

                                                            if (entry.is_dir)
                                                            {
                                                                // Navigate into directory
                                                                const newPath = path ? `${path}/${entry.filename}` : entry.filename;
                                                                await handleNavigate(newPath);
                                                            }
                                                            else if (isTextFile(entry.path))
                                                            {
                                                                // Toggle edit mode for text files
                                                                setIsEditingFile(prev => !prev);
                                                            }
                                                            else
                                                            {
                                                                // Download non-text, non-directory files
                                                                await downloadEntry(entry);
                                                            }
                                                        }}
                                                    >
                                                        <TableCell className={"flex items-center h-14 gap-2"}>
                                                            {isRenaming ? (
                                                                <Input
                                                                    defaultValue={entry.filename}
                                                                    autoFocus
                                                                    onBlur={e => renameSelectedEntry(e.currentTarget.value)}
                                                                    onKeyDown={async e =>
                                                                    {
                                                                        if (e.key === "Enter") await renameSelectedEntry(e.currentTarget.value);
                                                                    }}
                                                                    radius={"none"}
                                                                    className={"font-minecraft-body"}
                                                                />
                                                            ) : isNewItem ? (
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
                                                            ) : (
                                                                <>
                                                                    <FileEntryIcon entry={entry}/>
                                                                    <span className="flex-1">{entry.filename}</span>
                                                                    {isEditingFile && isSelected && isTextFile(entry.path) && (
                                                                        <div className="flex items-center gap-2 ml-auto">
                                                                            {needsToSave && (
                                                                                <Tooltip content="File has unsaved changes, press Ctrl+S to save">
                                                                                    <div className="w-2 h-2 rounded-full bg-warning animate-pulse"/>
                                                                                </Tooltip>
                                                                            )}
                                                                            {isExternallyModified && (
                                                                                <Tooltip content="File has been modified externally">
                                                                                    <Button
                                                                                        size="sm"
                                                                                        isIconOnly
                                                                                        radius="none"
                                                                                        variant="flat"
                                                                                        color="warning"
                                                                                        onPress={handleRefreshFileContents}
                                                                                        className="min-w-6 h-6"
                                                                                    >
                                                                                        <Icon icon="pixelarticons:alert" className="text-sm"/>
                                                                                    </Button>
                                                                                </Tooltip>
                                                                            )}
                                                                            <Tooltip content="Refresh file contents">
                                                                                <Button
                                                                                    size="sm"
                                                                                    isIconOnly
                                                                                    radius="none"
                                                                                    variant="flat"
                                                                                    onPress={handleRefreshFileContents}
                                                                                    className="min-w-6 h-6"
                                                                                >
                                                                                    <Icon icon="pixelarticons:reload" className="text-sm"/>
                                                                                </Button>
                                                                            </Tooltip>
                                                                            <Tooltip content="Save file (Ctrl+S)">
                                                                                <Button
                                                                                    size="sm"
                                                                                    isIconOnly
                                                                                    radius="none"
                                                                                    variant="flat"
                                                                                    color={needsToSave ? "primary" : "default"}
                                                                                    isDisabled={!needsToSave}
                                                                                    onPress={saveContent}
                                                                                    className="min-w-6 h-6"
                                                                                >
                                                                                    <Icon icon="pixelarticons:save" className="text-sm"/>
                                                                                </Button>
                                                                            </Tooltip>
                                                                        </div>
                                                                    )}
                                                                </>
                                                            )}
                                                        </TableCell>
                                                        <TableCell className={"text-gray-500"} hidden={isEditingFile && selectedEntries.length === 1}>
                                                            {entry.file_type}
                                                        </TableCell>
                                                        <TableCell className={"text-gray-500"} hidden={isEditingFile && selectedEntries.length === 1}>
                                                            {entry.is_dir ? "-" : Math.convertToByteString(entry.size)}
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
                                                );
                                            })}
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
                    onExtract={handleExtract}
                    onEdit={() =>
                    {
                        setIsEditingFile(true);
                        setContextMenuOptions(prev => ({...prev, isOpen: false}));
                    }}
                    onClose={() => setContextMenuOptions(prev => ({...prev, isOpen: false}))}
                />
            </div>

                <ServerFileEditor
                    ref={serverFileEditorRef}
                    isEditingFile={isEditingFile}
                    selectedEntries={selectedEntries}
                    selectedFileContents={selectedFileContents}
                    browserWidth={browserWidth}
                    containerRef={containerRef}
                    isDragging={isDragging}
                    needsToSave={needsToSave}
                    onContentChange={handleContentChange}
                    onSave={saveContent}
                    onWidthChange={handleWidthChange}
                    onDragStart={handleDragStart}
                    onDragEnd={handleDragEnd}
                />
            </div>

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
