import {createContext, ReactNode, useCallback, useContext, useRef, useState} from "react";
import $ from "jquery";
import {FileSystem, FilesystemData, FilesystemEntry} from "../ts/filesystem.ts";

export type Server =
    {
        /** Unique identifier for the server */
        id: string;
        /** Directory name where server files are stored, e.g. 'my_minecraft_server' */
        directory: string;
        /** Additional JVM arguments excluding -Xmx and -Xms */
        java_args: string;
        /** Maximum memory in GB for JVM -Xmx argument */
        max_memory: number;
        /** Minimum memory in GB for JVM -Xms argument */
        min_memory: number;
        /** Additional Minecraft server arguments */
        minecraft_args: string;
        /** Name/path of the server JAR file */
        server_jar: string;
        /** Whether UPnP port forwarding is enabled */
        upnp: boolean;
        /** Server status: 'stopped', 'starting', 'running', 'stopping', 'error' */
        status: ServerStatus;
        /** Whether the server should start automatically on boot */
        auto_start: boolean;
        /** Whether the server should restart automatically if it crashes */
        auto_restart: boolean;
        /** Whether automatic backups are enabled */
        backup_enabled: boolean;
        /** Backup interval in minutes */
        backup_interval: number;
        /** Optional server description */
        description: string | null;
        /** ID of the user who owns this server */
        owner_id: number;
        /** Timestamp of when the server was created (seconds since epoch) */
        created_at: number;
        /** Timestamp of when the server was last updated (seconds since epoch) */
        updated_at: number;
        /** Timestamp of when the server was last started (seconds since epoch) */
        last_started: number | null;
    } & CreateServerData

export type CreateServerData = {
    /** Name of the server, e.g. 'My Minecraft Server' */
    name: string;
    /** Server type: 'vanilla', 'fabric', 'forge', 'neoforge', 'quilt' , or 'custom' */
    server_type: LoaderType;
    /** Minecraft version, e.g. '1.20.1', '1.19.4', or 'custom' */
    minecraft_version: string;
    /** Loader version e.g. '0.14.0', '1.20.1-44.1.23', or 'custom' */
    loader_version: string;
    /** Path to Java executable, e.g. '/usr/bin/java' or 'java' for system PATH */
    java_executable: string;
}

export type LoaderType = "vanilla" | "fabric" | "forge" | "neoforge" | "quilt" | "custom";
export type ServerStatus = "idle" | "running" | "stopped" | "error" | "starting" | "stopping" | "crashed" | "hanging";

interface ServerContextType
{
    server: Server | null;
    servers: Server[];
    loadServer: (id: string) => Promise<void>;
    unloadServer: () => void;
    loadServers: () => Promise<void>;
    createServer: (server: CreateServerData) => Promise<string>;
    updateServer: (server: Partial<Server>, serverId?: string) => Promise<void>;
    deleteServer: (serverId?: string) => Promise<void>;
    startServer: (serverId?: string) => Promise<void>;
    stopServer: (serverId?: string) => Promise<void>;
    restartServer: (serverId?: string) => Promise<void>;
    killServer: (serverId?: string) => Promise<void>;
    sendCommand: (command: string, serverId?: string) => Promise<void>;
    subscribeToConsole: (callback: (data: string) => void, serverId?: string) => () => void;
    cleanupConsoleConnection: (serverId?: string) => void;
    hasActiveConsoleConnection: (serverId?: string) => boolean;
    backupServer: (serverId?: string) => Promise<void>;
    getServerStatus: (serverId?: string) => Promise<string>;
    isServerRunning: (serverId?: string) => boolean;
    // Filesystem functions
    getEntries: (path: string, serverId?: string) => Promise<FilesystemData>;
    downloadEntry: (entry: FilesystemEntry | FilesystemEntry[] | string | string[], serverId?: string) => Promise<void>;
    copyEntry: (sourcePaths: string[], destinationPath: string, serverId?: string) => Promise<void>;
    moveEntry: (sourcePaths: string[], destinationPath: string, serverId?: string) => Promise<void>;
    renameEntry: (source: string, destination: string, serverId?: string) => Promise<void>;
    deleteEntry: (path: string | string[], serverId?: string) => Promise<void>;
    uploadFile: (file: File, path: string, updateProgress: (bytes: number) => void, onCancelled?: () => void, serverId?: string) => Promise<{ promise: Promise<void>, cancel: () => Promise<void>, uploadId: string }>;
    createEntry: (filename: string, cwd: string, isDirectory: boolean, serverId?: string) => Promise<void>;
    searchFiles: (query: string, filename_only: boolean, abortSignal: AbortSignal, serverId?: string) => Promise<FilesystemEntry[]>;
    archiveFiles: (filename: string, filenames: string[], cwd: string, on_progress: (progress: number) => void, on_success: () => void, on_error: (msg: string) => void, on_cancelled?: () => void, serverId?: string) => { cancel: () => Promise<void>, trackerId: string };
    cancelArchive: (trackerId: string, serverId?: string) => Promise<void>;
    extractArchive: (archivePath: string, outputPath: string, on_progress: (progress: number, filesProcessed: number, totalFiles: number) => void, on_success: () => void, on_error: (msg: string) => void, on_cancelled?: () => void, serverId?: string) => { cancel: () => Promise<void>, trackerId: string };
    cancelExtract: (trackerId: string, serverId?: string) => Promise<void>;
    uploadFromUrl: (url: string, filepath: string, onProgress: (progress: number, downloaded: number, total: number) => void, onSuccess: () => void, onError: (error: string) => void, serverId?: string) => Promise<void>;
    getFileContents: (path: string, serverId?: string) => Promise<string>;
    setFileContents: (path: string, contents: string, serverId?: string) => Promise<void>;
    // Logging and console functions
    getLogs: (serverId?: string) => Promise<string[]>;
    getLog: (filename: string, serverId?: string) => Promise<string>;
}

const ServerContext = createContext<ServerContextType | undefined>(undefined);

export function ServerProvider({children}: { children: ReactNode })
{
    const [server, setServer] = useState<Server | null>(null);
    const [servers, setServers] = useState<Server[]>([]);

    // Connection tracking for console subscriptions
    const consoleConnections = useRef<Map<string, () => void>>(new Map());

    const loadServer = async (id: string) =>
    {
        let server: Server = await $.get(`/api/server/${id}`);
        server.status = server.status.toLowerCase() as ServerStatus; // Ensure server_type is lowercase
        setServer(server);
    };

    const unloadServer = () =>
    {
        setServer(null);
    };

    const loadServers = async () =>
    {
        let servers: Server[] = await $.get("/api/server");
        servers = servers.map(s => ({...s, status: s.status.toLowerCase()} as Server));
        setServers(servers);
    };

    const createServer = async (server: CreateServerData): Promise<string> =>
    {
        // Example response: { "message": "Server created successfully","server_id": "lW97O03zR32QygKY" }
        let response = await $.ajax({
            url: "/api/server",
            type: "PUT",
            contentType: "application/json",
            data: JSON.stringify(server)
        });
        if (!response || !response.server_id)
        {
            throw new Error("Server creation failed");
        }

        // Refresh servers list
        await loadServers();
        return response.server_id;
    };

    const isServerRunning = useCallback((serverId?: string): boolean =>
    {
        let targetServer = serverId ? servers.find(s => s.id === serverId) : server;
        if (!targetServer) return false;
        return targetServer.status === "running" || targetServer.status === "starting" || targetServer.status === "stopping" || targetServer.status === "hanging";
    }, [server, servers]);

    const updateServer = useCallback(async (updates: Partial<Server>, serverId?: string) =>
    {
        let targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");
        let targetServer: Server | null | undefined = serverId != undefined ? servers.find(s => s.id === targetServerId) : server;
        if (targetServer == null)
        {
            targetServer = await $.get(`/api/server/${targetServerId}`);
            if (!targetServer) throw new Error("No server loaded");
        }

        console.log("Updating server", targetServerId, updates, "Original server:", targetServer);
        const updatedServer = {...targetServer, ...updates};
        // the server status must be capitalized
        // `Idle`, `Running`, `Stopped`, `Error`, `Starting`, `Stopping`, `Crashed`, `Hanging`
        updatedServer.status = updatedServer.status.charAt(0).toUpperCase() + updatedServer.status.slice(1).toLowerCase() as ServerStatus;
        await $.ajax({
            url: `/api/server/${targetServerId}`,
            type: "POST",
            contentType: "application/json",
            data: JSON.stringify(updatedServer)
        });
        await loadServers();
    }, [server, servers]);

    const deleteServer = useCallback(async (serverId?: string) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        await $.ajax({
            url: `/api/server/${targetServerId}`,
            type: "DELETE"
        });

        // If deleting the currently loaded server, clear it
        if (server && server.id === targetServerId)
        {
            setServer(null);
        }

        // Remove from servers list
        setServers(prev => prev.filter(s => s.id !== targetServerId));
    }, [server]);

    const startServer = useCallback(async (serverId?: string) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        await $.post(`/api/server/${targetServerId}/start`);

        // Update status for currently loaded server if it matches
        if (server && server.id === targetServerId)
        {
            setServer(prev => prev ? {...prev, status: "starting"} : null);
        }
    }, [server]);

    const stopServer = useCallback(async (serverId?: string) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        await $.post(`/api/server/${targetServerId}/stop`);

        // Update status for currently loaded server if it matches
        if (server && server.id === targetServerId)
        {
            setServer(prev => prev ? {...prev, status: "stopping"} : null);
        }
    }, [server]);

    const restartServer = useCallback(async (serverId?: string) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        await $.post(`/api/server/${targetServerId}/restart`);

        // Update status for currently loaded server if it matches
        if (server && server.id === targetServerId)
        {
            setServer(prev => prev ? {...prev, status: "stopping"} : null);
        }

    }, [server]);

    const killServer = useCallback(async (serverId?: string) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        await $.post(`/api/server/${targetServerId}/kill`);

        // Update status for currently loaded server if it matches
        if (server && server.id === targetServerId)
        {
            setServer(prev => prev ? {...prev, status: "stopping"} : null);
        }

    }, [server]);

    const sendCommand = useCallback(async (command: string, serverId?: string) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        await $.ajax({
            url: `/api/server/${targetServerId}/send-command`,
            type: "POST",
            contentType: "text/plain",
            data: command
        });
    }, [server]);

    const cleanupConsoleConnection = useCallback((serverId?: string) =>
    {
        let targetServer = serverId ? servers.find(s => s.id === serverId) : server;
        const targetServerId = serverId || targetServer?.id;
        if (!targetServerId) return;

        const existingCleanup = consoleConnections.current.get(targetServerId);
        if (existingCleanup)
        {
            existingCleanup();
            consoleConnections.current.delete(targetServerId);
        }
    }, [server]);

    const hasActiveConsoleConnection = useCallback((serverId?: string): boolean =>
    {
        let targetServer = serverId ? servers.find(s => s.id === serverId) : server;
        const targetServerId = targetServer?.id;
        if (!targetServerId) return false;

        return consoleConnections.current.has(targetServerId);
    }, [server]);

    const subscribeToConsole = useCallback((callback: (data: string) => void, serverId?: string): (() => void) =>
    {
        let targetServer = serverId ? servers.find(s => s.id === serverId) : server;
        const targetServerId = targetServer?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        cleanupConsoleConnection(targetServerId);

        try
        {
            const eventSource = new EventSource(`/api/server/${targetServerId}/console`);
            const eventName = `console`;

            const handleMessage = (event: MessageEvent) =>
            {
                console.log(`Received console message for server ${targetServerId}:`, event.data);
                callback(event.data);
            };

            const handleError = (event: Event) =>
            {
                console.error(`EventSource error for server ${targetServerId}:`, event);
            };

            const handleOpen = () =>
            {
                console.log(`Console EventSource for server ${targetServerId} opened successfully.`);
            };

            eventSource.addEventListener("open", handleOpen);
            eventSource.addEventListener(eventName, handleMessage);
            eventSource.addEventListener("message", handleMessage);
            eventSource.addEventListener("error", handleError);

            const cleanup = () =>
            {
                eventSource.removeEventListener(eventName, handleMessage);
                eventSource.removeEventListener("message", handleMessage);
                eventSource.removeEventListener("error", handleError);
                eventSource.close();
                consoleConnections.current.delete(targetServerId);
                console.log(`EventSource for server ${targetServerId} closed and cleaned up.`);
            };

            // Store cleanup function in connection map
            consoleConnections.current.set(targetServerId, cleanup);


            // Return cleanup function
            return cleanup;
        } catch (error)
        {
            console.error(`Failed to create EventSource for server ${targetServerId}:`, error);
            throw error;
        }
    }, [server, cleanupConsoleConnection]);

    const backupServer = useCallback(async (serverId?: string) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        // Note: There's no backup endpoint in the server_endpoint.rs file
        // You may need to implement this endpoint on the backend
        throw new Error("Backup endpoint not implemented");
    }, [server]);

    const getServerStatus = useCallback(async (serverId?: string): Promise<string> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        const serverData: Server = await $.get(`/api/server/${targetServerId}`);
        return serverData.status;
    }, [server]);

    // Filesystem functions
    const getEntries = useCallback(async (path: string, serverId?: string): Promise<FilesystemData> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.getEntries(path, targetServerId);
    }, [server]);

    const downloadEntry = useCallback(async (entry: FilesystemEntry | FilesystemEntry[] | string | string[], serverId?: string): Promise<void> =>
    {
        const targetServer = serverId ? servers.find(s => s.id === serverId) : server;
        const targetServerId = targetServer?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");
        // if the entry is FileSystemEntry or FilesystemEntry[], convert it to string[]
        if (Array.isArray(entry))
        {
            if (entry[0] && typeof entry[0] === "object")
            {
                entry = (entry as FilesystemEntry[]).map(e => e.path);
            }
        } else if (typeof entry === "object")
        {
            entry = [(entry as FilesystemEntry).path];
        }

        return await FileSystem.download(entry as string[], targetServerId);
    }, [server]);

    const copyEntry = useCallback(async (sourcePaths: string[], destinationPath: string, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.copyEntry(sourcePaths, destinationPath, targetServerId);
    }, [server]);

    const moveEntry = useCallback(async (sourcePaths: string[], destinationPath: string, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.moveEntry(sourcePaths, destinationPath, targetServerId);
    }, [server]);

    const renameEntry = useCallback(async (source: string, destination: string, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.renameEntry(source, destination, targetServerId);
    }, [server]);

    const deleteEntry = useCallback(async (path: string | string[], serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.deleteEntry(path, targetServerId);
    }, [server]);

    const uploadFile = useCallback(async (file: File, path: string, updateProgress: (bytes: number) => void, onCancelled?: () => void, serverId?: string): Promise<{ promise: Promise<void>, cancel: () => Promise<void>, uploadId: string }> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.upload(file, path, targetServerId, updateProgress, onCancelled);
    }, [server]);

    const createEntry = useCallback(async (filename: string, cwd: string, isDirectory: boolean, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.createEntry(filename, cwd, isDirectory, targetServerId);
    }, [server]);

    const searchFiles = useCallback(async (query: string, filename_only: boolean, abortSignal: AbortSignal, serverId?: string): Promise<FilesystemEntry[]> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.search(query, filename_only, targetServerId, abortSignal);
    }, [server]);

    const archiveFiles = useCallback((filename: string, filenames: string[], cwd: string, on_progress: (progress: number) => void, on_success: () => void, on_error: (msg: string) => void, on_cancelled?: () => void, serverId?: string): { cancel: () => Promise<void>, trackerId: string } =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return FileSystem.archive(filename, filenames, cwd, targetServerId, on_progress, on_success, on_error, on_cancelled);
    }, [server]);

    const cancelArchive = useCallback(async (trackerId: string, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.cancelArchive(trackerId, targetServerId);
    }, [server]);

    const extractArchive = useCallback((archivePath: string, outputPath: string, on_progress: (progress: number, filesProcessed: number, totalFiles: number) => void, on_success: () => void, on_error: (msg: string) => void, on_cancelled?: () => void, serverId?: string): { cancel: () => Promise<void>, trackerId: string } =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return FileSystem.extract(archivePath, outputPath, targetServerId, on_progress, on_success, on_error, on_cancelled);
    }, [server]);

    const cancelExtract = useCallback(async (trackerId: string, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.cancelExtract(trackerId, targetServerId);
    }, [server]);

    const uploadFromUrl = useCallback(async (url: string, filepath: string, onProgress: (progress: number, downloaded: number, total: number) => void, onSuccess: () => void, onError: (error: string) => void, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.uploadFromUrl(url, filepath, targetServerId, onProgress, onSuccess, onError);
    }, [server]);

    const getLogs = useCallback(async (serverId?: string): Promise<string[]> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return $.get(`/api/server/${targetServerId}/logs`);
    }, [server]);

    const getLog = useCallback(async (filename: string, serverId?: string): Promise<string> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return $.get(`/api/server/${targetServerId}/logs/${filename}`);
    }, [server]);
    
    const getFileContents = useCallback(async (path: string, serverId?: string): Promise<string> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.getFileContents(path, targetServerId);
    }, [server]);
    
    const setFileContents = useCallback(async (path: string, contents: string, serverId?: string): Promise<void> =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        return await FileSystem.setFileContents(path, contents, targetServerId);
    }, [server]);


    return (
        <ServerContext.Provider value={{
            server,
            servers,
            loadServer,
            unloadServer,
            loadServers,
            createServer,
            updateServer,
            deleteServer,
            startServer,
            stopServer,
            restartServer,
            killServer,
            sendCommand,
            subscribeToConsole,
            cleanupConsoleConnection,
            hasActiveConsoleConnection,
            backupServer,
            getServerStatus,
            isServerRunning,
            // Filesystem functions
            getEntries,
            downloadEntry,
            copyEntry,
            moveEntry,
            renameEntry,
            deleteEntry,
            uploadFile,
            createEntry,
            searchFiles,
            archiveFiles,
            cancelArchive,
            extractArchive,
            cancelExtract,
            uploadFromUrl,
            getFileContents,
            setFileContents,
            getLogs,
            getLog
        }}>
            {children}
        </ServerContext.Provider>
    );
}

export function useServer(): ServerContextType
{
    const context = useContext(ServerContext);
    if (!context)
    {
        throw new Error("useServer must be used within a ServerProvider");
    }
    return context;
}