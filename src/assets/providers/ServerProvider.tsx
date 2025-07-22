import {createContext, ReactNode, useCallback, useContext, useState} from "react";
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
    loadServer: (id: string) => Promise<void>;
    createServer: (server: CreateServerData) => Promise<void>;
    updateServer: (server: Partial<Server>) => Promise<void>;
    deleteServer: () => Promise<void>;
    startServer: () => Promise<void>;
    stopServer: () => Promise<void>;
    restartServer: () => Promise<void>;
    backupServer: () => Promise<void>;
    getServerStatus: () => Promise<string>;
}

const ServerContext = createContext<ServerContextType | undefined>(undefined);

export function ServerProvider({children}: { children: ReactNode })
{
    const [server, setServer] = useState<Server | null>(null);

    const loadServer = async (id: string) =>
    {
        let server: Server = await $.get(`/api/server/${id}`);
        setServer(server);
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
    };
    const startServer = useCallback(async () =>
    {
        if (!server) throw new Error("No server loaded");
        await $.post(`/api/server/${server.id}/start`);
        setServer(prev => prev ? {...prev, status: "starting"} : null);
        let id = server.id;

        let statusPoll = setInterval(async () =>
        {
            let server: Server = await $.get(`/api/server/${id}`);
            if (server.status === "running" || server.status === "error" || server.status === "crashed")
            {
                setServer(server);
                clearInterval(statusPoll);
            }

        }, 1000);

    }, [server]);


    return (
        <ServerContext.Provider value={{server, loadServer, createServer, startServer}}>
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