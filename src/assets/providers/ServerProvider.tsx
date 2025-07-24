import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import $ from "jquery";

export type Server =
    {
        /** Unique identifier for the server */
        id: string;
        /** Name of the server, e.g. 'My Minecraft Server' */
        name: string;
        /** Directory name where server files are stored, e.g. 'my_minecraft_server' */
        directory: string;
        /** Path to Java executable, e.g. '/usr/bin/java' or 'java' for system PATH */
        java_executable: string | null;
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
        status: "stopped" | "starting" | "running" | "stopping" | "error" | "crashed";
        /** Whether server should start automatically on boot */
        auto_start: boolean;
        /** Whether server should restart automatically if it crashes */
        auto_restart: boolean;
        /** Whether automatic backups are enabled */
        backup_enabled: boolean;
        /** Backup interval in minutes */
        backup_interval: number;
        /** Optional server description */
        description: string | null;
        /** Minecraft version, e.g. '1.20.1', '1.19.4', or 'custom' */
        minecraft_version: string | null;
        /** Server type: 'vanilla', 'fabric', 'forge', 'neoforge', 'quilt', or 'custom' */
        server_type: "vanilla" | "fabric" | "forge" | "neoforge" | "quilt" | "custom" | null;
        /** Loader version e.g. '0.14.0', '1.20.1-44.1.23', or 'custom' */
        loader_version: string | null;
        /** ID of the user who owns this server */
        owner_id: number;
        /** Timestamp of when the server was created (seconds since epoch) */
        created_at: number;
        /** Timestamp of when the server was last updated (seconds since epoch) */
        updated_at: number;
        /** Timestamp of when the server was last started (seconds since epoch) */
        last_started: number | null;
    }

export type CreateServerData = {
    name: string;
    server_type: "vanilla" | "fabric" | "forge" | "neoforge" | "quilt" | "custom";
    minecraft_version: string;
    loader_version: string;
}

interface ServerContextType
{
    server: Server | null;
    servers: Server[];
    loadServer: (id: string) => Promise<void>;
    loadServers: () => Promise<void>;
    createServer: (server: CreateServerData) => Promise<void>;
    updateServer: (server: Partial<Server>) => Promise<void>;
    deleteServer: (serverId?: string) => Promise<void>;
    startServer: (serverId?: string) => Promise<void>;
    stopServer: (serverId?: string) => Promise<void>;
    restartServer: (serverId?: string) => Promise<void>;
    killServer: (serverId?: string) => Promise<void>;
    sendCommand: (command: string, serverId?: string) => Promise<void>;
    subscribeToConsole: (callback: (data: string) => void, serverId?: string) => () => void;
    backupServer: (serverId?: string) => Promise<void>;
    getServerStatus: (serverId?: string) => Promise<string>;
}

const ServerContext = createContext<ServerContextType | undefined>(undefined);

export function ServerProvider({children}: { children: ReactNode })
{
    const [server, setServer] = useState<Server | null>(null);
    const [servers, setServers] = useState<Server[]>([]);

    const loadServer = async (id: string) =>
    {
        let server: Server = await $.get(`/api/server/${id}`);
        setServer(server);
    };

    const loadServers = async () =>
    {
        let servers: Server[] = await $.get("/api/server");
        setServers(servers);
    };

    const createServer = async (server: CreateServerData) =>
    {
        let newServer: Server = await $.ajax({
            url: "/api/server",
            type: "PUT",
            contentType: "application/json",
            data: JSON.stringify(server)
        });
        setServer(newServer);
        // Refresh servers list
        await loadServers();
    };

    const updateServer = useCallback(async (updates: Partial<Server>) =>
    {
        if (!server) throw new Error("No server loaded");

        const updatedServer = {...server, ...updates};
        await $.ajax({
            url: `/api/server/${server.id}`,
            type: "POST",
            contentType: "application/json",
            data: JSON.stringify(updatedServer)
        });

        setServer(updatedServer);
        // Update in servers list if present
        setServers(prev => prev.map(s => s.id === server.id ? updatedServer : s));
    }, [server]);

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

        let statusPoll = setInterval(async () =>
        {
            let updatedServer: Server = await $.get(`/api/server/${targetServerId}`);
            if (updatedServer.status === "running" || updatedServer.status === "error" || updatedServer.status === "crashed")
            {
                // Update currently loaded server if it matches
                if (server && server.id === targetServerId)
                {
                    setServer(updatedServer);
                }
                setServers(prev => prev.map(s => s.id === targetServerId ? updatedServer : s));
                clearInterval(statusPoll);
            }
        }, 1000);
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

        let statusPoll = setInterval(async () =>
        {
            let updatedServer: Server = await $.get(`/api/server/${targetServerId}`);
            if (updatedServer.status === "stopped" || updatedServer.status === "error" || updatedServer.status === "crashed")
            {
                // Update currently loaded server if it matches
                if (server && server.id === targetServerId)
                {
                    setServer(updatedServer);
                }
                setServers(prev => prev.map(s => s.id === targetServerId ? updatedServer : s));
                clearInterval(statusPoll);
            }
        }, 1000);
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

        let statusPoll = setInterval(async () =>
        {
            let updatedServer: Server = await $.get(`/api/server/${targetServerId}`);
            if (updatedServer.status === "running" || updatedServer.status === "error" || updatedServer.status === "crashed")
            {
                // Update currently loaded server if it matches
                if (server && server.id === targetServerId)
                {
                    setServer(updatedServer);
                }
                setServers(prev => prev.map(s => s.id === targetServerId ? updatedServer : s));
                clearInterval(statusPoll);
            }
        }, 1000);
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

        let statusPoll = setInterval(async () =>
        {
            let updatedServer: Server = await $.get(`/api/server/${targetServerId}`);
            if (updatedServer.status === "stopped" || updatedServer.status === "error" || updatedServer.status === "crashed")
            {
                // Update currently loaded server if it matches
                if (server && server.id === targetServerId)
                {
                    setServer(updatedServer);
                }
                setServers(prev => prev.map(s => s.id === targetServerId ? updatedServer : s));
                clearInterval(statusPoll);
            }
        }, 1000);
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

    const subscribeToConsole = useCallback((callback: (data: string) => void, serverId?: string): (() => void) =>
    {
        const targetServerId = serverId || server?.id;
        if (!targetServerId) throw new Error("No server ID provided and no server loaded");

        const eventSource = new EventSource(`/api/server/${targetServerId}/console`);
        const eventName = `server-${targetServerId}-console`;

        const handleMessage = (event: MessageEvent) =>
        {
            callback(event.data);
        };

        eventSource.addEventListener(eventName, handleMessage);

        // Return cleanup function
        return () =>
        {
            eventSource.removeEventListener(eventName, handleMessage);
            eventSource.close();
        };
    }, [server]);

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

    useEffect(() =>
    {
        // Load servers on initial mount
        loadServers().catch(console.error);

        // If a server is already loaded, load its details
        if (server)
        {
            loadServer(server.id).catch(console.error);
        }
    }, []);

    return (
        <ServerContext.Provider value={{
            server,
            servers,
            loadServer,
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
            backupServer,
            getServerStatus
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