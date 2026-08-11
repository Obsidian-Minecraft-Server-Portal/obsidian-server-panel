import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {addToast} from "@heroui-compat";
import {NeoForgeVersionList, getNeoForgeVersions} from "../../ts/neoforge-versions.ts";
import {useAuthentication} from "../AuthenticationProvider.tsx";

interface NeoForgeVersionsContextType
{
    neoforgeVersions: NeoForgeVersionList | null;
    getFromMinecraftVersion: (minecraftVersion: string) => string[];
    refreshNeoForgeVersions: () => Promise<void>;
}

const NeoForgeVersionsContext = createContext<NeoForgeVersionsContextType | undefined>(undefined);

export function NeoForgeVersionsProvider({children}: { children: ReactNode })
{
    const {isAuthenticated} = useAuthentication();
    const [neoforgeVersions, setNeoForgeVersions] = useState<NeoForgeVersionList | null>(null);

    const refreshNeoForgeVersions = useCallback(async () =>
    {
        setNeoForgeVersions(await getNeoForgeVersions());
    }, []);

    useEffect(() =>
    {
        if (!isAuthenticated) return;

        refreshNeoForgeVersions()
            .then(() => console.log("Loaded neoforge versions successfully."))
            .catch(error =>
            {
                console.error("Failed to load neoforge versions:", error);
                setNeoForgeVersions(null); // Reset to null on error
                addToast({
                    title: "Error",
                    description: "Failed to load neoforge versions. Please try again later.",
                    color: "danger"
                });
            });
    }, [isAuthenticated]);

    const getFromMinecraftVersion = useCallback((minecraftVersion: string): string[] =>
    {
        if (!neoforgeVersions) return [];
        const parts = minecraftVersion.split(".");
        // NeoForge drops the legacy "1." prefix: 1.21.4 -> 21.4.x, 1.21 -> 21.0.x, 26.2 -> 26.2.x
        const [major, minor] = parts[0] === "1" ? [parts[1], parts[2]] : [parts[0], parts[1]];
        if (!major) return [];
        const prefix = `${major}.${minor ?? "0"}.`;
        return neoforgeVersions.versions.filter(version => version.startsWith(prefix));
    }, [neoforgeVersions]);

    return (
        <NeoForgeVersionsContext.Provider value={{neoforgeVersions, refreshNeoForgeVersions, getFromMinecraftVersion}}>
            {children}
        </NeoForgeVersionsContext.Provider>
    );
}

export function useNeoForgeVersions(): NeoForgeVersionsContextType
{
    const context = useContext(NeoForgeVersionsContext);
    if (!context)
    {
        throw new Error("useNeoForgeVersions must be used within a NeoForgeVersionsProvider");
    }
    return context;
}