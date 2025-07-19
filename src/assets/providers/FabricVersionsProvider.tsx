import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {addToast} from "@heroui/react";
import {FabricVersionList, getFabricVersions} from "../ts/fabric-versions.ts";

interface FabricVersionsContextType
{
    fabricVersions: FabricVersionList | null;
    refreshFabricVersions: () => Promise<void>;
}

const FabricVersionsContext = createContext<FabricVersionsContextType | undefined>(undefined);

export function FabricVersionsProvider({children}: { children: ReactNode })
{
    const [fabricVersions, setFabricVersions] = useState<FabricVersionList | null>(null);

    const refreshFabricVersions = useCallback(async () =>
    {
        setFabricVersions(await getFabricVersions());
    }, []);

    useEffect(() =>
    {
        refreshFabricVersions()
            .then(() => console.log("Loaded fabric versions successfully."))
            .catch(error =>
            {
                console.error("Failed to load fabric versions:", error);
                setFabricVersions(null); // Reset to null on error
                addToast({
                    title: "Error",
                    description: "Failed to load fabric versions. Please try again later.",
                    color: "danger"
                });
            });
    }, []);

    return (
        <FabricVersionsContext.Provider value={{fabricVersions, refreshFabricVersions}}>
            {children}
        </FabricVersionsContext.Provider>
    );
}

export function useFabricVersions(): FabricVersionsContextType
{
    const context = useContext(FabricVersionsContext);
    if (!context)
    {
        throw new Error("useFabricVersions must be used within a FabricVersionsProvider");
    }
    return context;
}