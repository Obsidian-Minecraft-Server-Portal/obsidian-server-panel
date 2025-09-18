import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {addToast} from "@heroui/react";
import {NeoForgeVersionList, getNeoForgeVersions} from "../../ts/neoforge-versions.ts";

interface NeoForgeVersionsContextType
{
    neoforgeVersions: NeoForgeVersionList | null;
    getFromMinecraftVersion: (minecraftVersion: string) => string[];
    refreshNeoForgeVersions: () => Promise<void>;
}

const NeoForgeVersionsContext = createContext<NeoForgeVersionsContextType | undefined>(undefined);

export function NeoForgeVersionsProvider({children}: { children: ReactNode })
{
    const [neoforgeVersions, setNeoForgeVersions] = useState<NeoForgeVersionList | null>(null);

    const refreshNeoForgeVersions = useCallback(async () =>
    {
        setNeoForgeVersions(await getNeoForgeVersions());
    }, []);

    useEffect(() =>
    {
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
    }, []);

    const getFromMinecraftVersion = useCallback((minecraftVersion: string): string[] =>
    {
        if (!neoforgeVersions) return [];
        let start = minecraftVersion.substring(2); // Remove the "1." prefix
        return neoforgeVersions.versions.filter(version => version.startsWith(start));
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