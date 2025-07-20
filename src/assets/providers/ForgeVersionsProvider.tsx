import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {addToast} from "@heroui/react";
import {ForgeVersionList, getForgeVersions} from "../ts/forge-versions.ts";
import {useAuthentication} from "./AuthenticationProvider.tsx";

interface ForgeVersionsContextType
{
    forgeVersions: ForgeVersionList | null;
    refreshForgeVersions: () => Promise<void>;
}

const ForgeVersionsContext = createContext<ForgeVersionsContextType | undefined>(undefined);

export function ForgeVersionsProvider({children}: { children: ReactNode })
{
    const {isAuthenticated} = useAuthentication();
    const [forgeVersions, setForgeVersions] = useState<ForgeVersionList | null>(null);

    const refreshForgeVersions = useCallback(async () =>
    {
        setForgeVersions(await getForgeVersions());
    }, []);

    useEffect(() =>
    {
        if (!isAuthenticated) return;

        refreshForgeVersions()
            .then(() => console.log("Loaded forge versions successfully."))
            .catch(error =>
            {
                console.error("Failed to load forge versions:", error);
                setForgeVersions(null); // Reset to null on error
                addToast({
                    title: "Error",
                    description: "Failed to load forge versions. Please try again later.",
                    color: "danger"
                });
            });
    }, [isAuthenticated]);

    return (
        <ForgeVersionsContext.Provider value={{forgeVersions, refreshForgeVersions}}>
            {children}
        </ForgeVersionsContext.Provider>
    );
}

export function useForgeVersions(): ForgeVersionsContextType
{
    const context = useContext(ForgeVersionsContext);
    if (!context)
    {
        throw new Error("useForgeVersions must be used within a ForgeVersionsProvider");
    }
    return context;
}