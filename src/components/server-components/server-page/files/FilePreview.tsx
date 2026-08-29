import {Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, cn} from "@heroui-compat";
import {Icon} from "@iconify-icon/react";
import {useServer} from "../../../../providers/ServerProvider.tsx";
import {Button} from "../../../extended/Button.tsx";
import {Input} from "../../../extended/Input.tsx";
import {useCallback, useEffect, useRef, useState} from "react";
import {FilesystemEntry} from "../../../../ts/filesystem.ts";
import {getFileType, isTextFile} from "../../../../ts/file-type-match.ts";

const IMAGE_EXTENSIONS = new Set(["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "avif"]);

const extensionMimeTypes: Record<string, string> = {
    png: "image/png",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    gif: "image/gif",
    webp: "image/webp",
    svg: "image/svg+xml",
    bmp: "image/bmp",
    ico: "image/x-icon",
    avif: "image/avif"
};

function formatBytes(bytes: number): string
{
    if (bytes < 1024) return `${bytes} B`;
    const units = ["KB", "MB", "GB", "TB"];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1)
    {
        value /= 1024;
        unit++;
    }
    return `${value.toFixed(value >= 100 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function getFileIcon(entry: FilesystemEntry): string
{
    if (entry.is_dir) return "pixelarticons:folder";
    const extension = entry.filename.split(".").pop()?.toLowerCase() ?? "";
    if (IMAGE_EXTENSIONS.has(extension)) return "pixelarticons:image";
    if (["zip", "tar", "gz", "tgz", "rar", "7z"].includes(extension)) return "pixelarticons:archive";
    if (extension === "jar") return "pixelarticons:box";
    if (["log", "txt", "md", "properties", "toml", "json", "yml", "yaml", "cfg", "conf"].includes(extension)) return "pixelarticons:file-text";
    return "pixelarticons:file";
}

export interface FilePreviewProps
{
    serverId: string;
    entry: FilesystemEntry;
    onRename: (entry: FilesystemEntry, newName: string) => Promise<void>;
    onDelete: (entries: FilesystemEntry[]) => void;
    onArchive: (entries: FilesystemEntry[]) => void;
    onExtract: (entry: FilesystemEntry, outputPath?: string) => void;
    onAddToIgnore: (entries: FilesystemEntry[]) => void;
    onEdit: (entry: FilesystemEntry) => void;
}

export function FilePreview({serverId, entry, onRename, onDelete, onArchive, onExtract, onAddToIgnore, onEdit}: FilePreviewProps)
{
    const {downloadEntry} = useServer();
    const [imagePreviewUrl, setImagePreviewUrl] = useState<string | null>(null);
    const [imageLoadFailed, setImageLoadFailed] = useState(false);
    const [nameDraft, setNameDraft] = useState(entry.filename);
    const [isRenaming, setIsRenaming] = useState(false);
    const nameInputRef = useRef<HTMLInputElement>(null);
    const objectUrlRef = useRef<string | null>(null);

    const isDirectory = entry.is_dir;
    const extension = entry.filename.split(".").pop()?.toLowerCase() ?? "";
    const isImage = !isDirectory && IMAGE_EXTENSIONS.has(extension);
    const isArchive = !isDirectory && ["zip", "tar", "gz", "tgz", "rar", "7z"].includes(extension);
    const fileType = getFileType(entry.path);

    // keep the rename input in sync when the selection changes
    useEffect(() =>
    {
        setNameDraft(entry.filename);
        setIsRenaming(false);
    }, [entry.path, entry.filename]);

    // fetch image previews as an authenticated blob
    useEffect(() =>
    {
        setImagePreviewUrl(null);
        setImageLoadFailed(false);
        if (!isImage) return;

        let cancelled = false;
        const controller = new AbortController();
        fetch(`/api/server/${serverId}/fs/contents?filepath=${encodeURIComponent(entry.path)}`, {
            credentials: "include",
            signal: controller.signal
        }).then(async response =>
        {
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            const buffer = await response.arrayBuffer();
            if (cancelled) return;
            const blob = new Blob([buffer], {type: extensionMimeTypes[extension] ?? "application/octet-stream"});
            const url = URL.createObjectURL(blob);
            objectUrlRef.current = url;
            setImagePreviewUrl(url);
        }).catch(error =>
        {
            if (!cancelled && (error as Error).name !== "AbortError")
            {
                console.error("Failed to load image preview:", error);
                setImageLoadFailed(true);
            }
        });

        return () =>
        {
            cancelled = true;
            controller.abort();
            if (objectUrlRef.current)
            {
                URL.revokeObjectURL(objectUrlRef.current);
                objectUrlRef.current = null;
            }
        };
    }, [serverId, entry.path, isImage, extension]);

    const commitRename = useCallback(async () =>
    {
        if (isRenaming) return;
        setIsRenaming(true);
        try
        {
            await onRename(entry, nameDraft);
        } finally
        {
            setIsRenaming(false);
        }
    }, [entry, nameDraft, onRename, isRenaming]);

    const startRename = useCallback(() =>
    {
        nameInputRef.current?.focus();
        nameInputRef.current?.select();
    }, []);

    const menuItems = [
        ...(isTextFile(entry.path) ? [{key: "edit", label: "Edit", icon: "pixelarticons:edit-box", action: () => onEdit(entry)}] : []),
        ...(isArchive ? [
            {key: "extract-here", label: "Extract Here", icon: "pixelarticons:open", action: () => onExtract(entry)},
            {key: "extract-to", label: `Extract to ${entry.filename.replace(/\.[^.]+$/, "")}`, icon: "pixelarticons:folder-plus", action: () => onExtract(entry, entry.filename.replace(/\.[^.]+$/, ""))}
        ] : []),
        {key: "archive", label: "Archive", icon: "pixelarticons:archive", action: () => onArchive([entry])},
        {key: "ignore", label: "Add to .obakignore", icon: "pixelarticons:eye-closed", action: () => onAddToIgnore([entry])},
        {key: "delete", label: "Delete", icon: "pixelarticons:trash", danger: true, action: () => onDelete([entry])}
    ];

    return (
        <div className={"flex flex-col items-center gap-6 w-full h-full overflow-y-auto p-6"}>
            {/* preview area */}
            <div className="flex items-center justify-center grow w-full min-h-32">
                {isImage && imagePreviewUrl && !imageLoadFailed && (
                    <img
                        src={imagePreviewUrl}
                        alt={entry.filename}
                        className="max-w-full max-h-full object-contain rounded-none border border-default-300"
                    />
                )}
                {isImage && !imagePreviewUrl && !imageLoadFailed && (
                    <Icon icon="pixelarticons:image" className="text-default-300 text-8xl" />
                )}
                {isImage && imageLoadFailed && (
                    <Icon icon="pixelarticons:image" className="text-default-300 text-8xl" />
                )}
                {!isImage && (
                    <Icon icon={getFileIcon(entry)} className="text-default-300 text-[120px]"/>
                )}
            </div>

            {/* file name with inline rename */}
            <div className="flex flex-col items-center gap-1 w-full max-w-md">
                <Input
                    ref={nameInputRef}
                    value={nameDraft}
                    onValueChange={setNameDraft}
                    onKeyDown={e =>
                    {
                        if (e.key === "Enter") void commitRename();
                        if (e.key === "Escape")
                        {
                            setNameDraft(entry.filename);
                            (e.target as HTMLInputElement).blur();
                        }
                    }}
                    onBlur={() => void commitRename()}
                    onFocus={e => e.currentTarget.select()}
                    label="File name"
                    classNames={{inputWrapper: "font-minecraft-body"}}
                    startContent={<Icon icon={getFileIcon(entry)} className="text-default-500"/>}
                    isDisabled={isRenaming}
                />
                <p className="text-xs text-default-500 font-minecraft-body">
                    {isDirectory ? "Folder" : fileType?.description ?? `${extension.toUpperCase() || "File"} file`}
                    {!isDirectory && ` - ${formatBytes(entry.size)}`}
                    {entry.last_modified && ` - modified ${new Date(entry.last_modified).toLocaleDateString()}`}
                </p>
                <p className="text-xs text-default-300 font-minecraft-body truncate max-w-full" title={entry.path}>{entry.path}</p>
            </div>

            {/* actions */}
            <div className="flex items-center gap-2">
                <Button
                    color="primary"
                    variant="solid"
                    size="sm"
                    startContent={<Icon icon="pixelarticons:download"/>}
                    onPress={() => void downloadEntry(entry)}
                >
                    Download
                </Button>
                <Button size="sm" variant="flat" startContent={<Icon icon="pixelarticons:edit"/>} onPress={startRename}>
                    Rename
                </Button>
                <Dropdown placement="bottom-end" className="rounded-none font-minecraft-body">
                    <DropdownTrigger>
                        <Button isIconOnly size="sm" variant="flat" aria-label="File actions">
                            <Icon icon="pixelarticons:more-vertical"/>
                        </Button>
                    </DropdownTrigger>
                    <DropdownMenu aria-label="File preview actions" className="rounded-none font-minecraft-body">
                        {menuItems.map(item => (
                            <DropdownItem
                                key={item.key}
                                startContent={<Icon icon={item.icon}/>}
                                className={cn("rounded-none font-minecraft-body", item.danger && "text-danger")}
                                onPress={item.action}
                            >
                                {item.label}
                            </DropdownItem>
                        ))}
                    </DropdownMenu>
                </Dropdown>
            </div>
        </div>
    );
}