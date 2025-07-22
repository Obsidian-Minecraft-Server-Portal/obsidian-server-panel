import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {getMinecraftVersions, MinecraftVersionList} from "../../ts/minecraft-versions.ts";
import {addToast} from "@heroui/react";
import {ForgeVersionsProvider} from "./ForgeVersionsProvider.tsx";
import {FabricVersionsProvider} from "./FabricVersionsProvider.tsx";
import {QuiltVersionsProvider} from "./QuiltVersionsProvider.tsx";
import {NeoForgeVersionsProvider} from "./NeoForgeVersionsProvider.tsx";

interface MinecraftVersionsContextType
{
    minecraftVersions: MinecraftVersionList | null;
    refreshMinecraftVersions: () => Promise<void>;
}

const MinecraftVersionsContext = createContext<MinecraftVersionsContextType | undefined>(undefined);

export function MinecraftVersionsProvider({children}: { children: ReactNode })
{
    const [minecraftVersions, setMinecraftVersions] = useState<MinecraftVersionList | null>(null);

    const refreshMinecraftVersions = useCallback(async () =>
    {
        setMinecraftVersions(await getMinecraftVersions());
    }, [setMinecraftVersions]);

    useEffect(() =>
    {
        refreshMinecraftVersions()
            .then(() => console.log("Loaded Minecraft versions successfully."))
            .catch(error =>
            {
                console.error("Failed to load Minecraft versions:", error);
                setMinecraftVersions(null); // Reset to null on error
                addToast({
                    title: "Error",
                    description: "Failed to load Minecraft versions. Please try again later.",
                    color: "danger"
                });
            });
    }, []);

    return (

        <MinecraftVersionsContext.Provider value={{minecraftVersions, refreshMinecraftVersions}}>
            <ForgeVersionsProvider>
                <FabricVersionsProvider>
                    <QuiltVersionsProvider>
                        <NeoForgeVersionsProvider>
                            {children}
                        </NeoForgeVersionsProvider>
                    </QuiltVersionsProvider>
                </FabricVersionsProvider>
            </ForgeVersionsProvider>
        </MinecraftVersionsContext.Provider>
    );
}

export function useMinecraftVersions(): MinecraftVersionsContextType
{
    const context = useContext(MinecraftVersionsContext);
    if (!context)
    {
        throw new Error("useMinecraftVersions must be used within a MinecraftVersionsProvider");
    }
    return context;
}