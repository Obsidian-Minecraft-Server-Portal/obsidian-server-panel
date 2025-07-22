import $ from "jquery";
import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";

export type HostInfo = {
    version: string,
    is_development: boolean,
    has_admin_user: boolean,
    resources: {
        os: string,
        num_cores: number,
        total_memory: number,
    }
}

export type ResourceData = {
    cpu_usage?: CpuUsage,
    allocated_memory?: number, // Memory allocated to the server in Bytes
    disk_usage?: RWUsage[], // Disk usage in read/write operations per second
    network_usage?: RWUsage[], // Network usage in read/write operations per second

}

export type CpuUsage = {
    total_usage: number; // Total CPU usage percentage
    cores: number[]; // Array of CPU usage percentages per core
}

export type RWUsage = {
    device: string; // Device name (e.g., /dev/sda)
    read: number; // Read operations per second
    write: number; // Write operations per second
    mtu?: number; // Maximum Transmission Unit in bytes
}

interface HostInfoContextType
{
    hostInfo: HostInfo;
    resources: ResourceData;
    refreshHostInfo: () => Promise<void>;
}

const HostInfoContext = createContext<HostInfoContextType | undefined>(undefined);

export function HostInfoProvider({children}: { children: ReactNode })
{
    const [hostInfo, setHostInfo] = useState<HostInfo>({resources: {total_memory: 1, num_cores: 1}} as HostInfo);
    const [resources, setResources] = useState<ResourceData>({} as ResourceData);

    const getHostInfo = useCallback(async () =>
    {
        setHostInfo(await $.get("/api/info"));
    }, [setHostInfo]);

    useEffect(() =>
    {
        getHostInfo().then(console.log);
        let connection = new EventSource("/api/info/resources", {withCredentials: true});
        connection.onopen = () =>
        {
            console.log("SSE connection established.");
            getHostInfo().then(console.log);
        };
        connection.addEventListener("resource_update", (event) =>
        {
            const data = JSON.parse(event.data);
            setResources(data);
        });
        connection.onerror = (error) =>
        {
            console.error("Error in SSE connection:", error);
            connection.close();
        };
        return () =>
        {
            connection.close();
        };
    }, []);

    return (
        <HostInfoContext.Provider value={{hostInfo, refreshHostInfo: getHostInfo, resources}}>
            {children}
        </HostInfoContext.Provider>
    );
}

export function useHostInfo(): HostInfoContextType
{
    const context = useContext(HostInfoContext);
    if (!context)
    {
        throw new Error("useServerInfo must be used within a HostInfoProvider");
    }
    return context;
}