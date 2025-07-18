import $ from "jquery";
import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";

export type ServerInfo = {
    version: string,
    is_development: boolean,
    has_admin_user: boolean,
}

interface ServerInfoContextType
{
    serverInfo: ServerInfo;
    refreshServerInfo: () => Promise<void>;
}

const ServerInfoContext = createContext<ServerInfoContextType | undefined>(undefined);

export function ServerInfoProvider({children}: { children: ReactNode })
{
    const [serverInfo, setServerInfo] = useState<ServerInfo>({} as ServerInfo);

    const getServerInfo = useCallback(async () =>
    {
        setServerInfo(await $.get("/api/"));
    }, [setServerInfo]);

    useEffect(() =>
    {
        getServerInfo().then(console.log);
    }, []);

    return (
        <ServerInfoContext.Provider value={{serverInfo, refreshServerInfo: getServerInfo}}>
            {children}
        </ServerInfoContext.Provider>
    );
}

export function useServerInfo(): ServerInfoContextType
{
    const context = useContext(ServerInfoContext);
    if (!context)
    {
        throw new Error("useServerInfo must be used within a ServerInfoProvider");
    }
    return context;
}