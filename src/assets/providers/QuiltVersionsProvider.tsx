import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {addToast} from "@heroui/react";
import {QuiltVersionList, getQuiltVersions} from "../ts/quilt-versions.ts";

interface QuiltVersionsContextType
{
    quiltVersions: QuiltVersionList | null;
    refreshQuiltVersions: () => Promise<void>;
}

const QuiltVersionsContext = createContext<QuiltVersionsContextType | undefined>(undefined);

export function QuiltVersionsProvider({children}: { children: ReactNode })
{
    const [quiltVersions, setQuiltVersions] = useState<QuiltVersionList | null>(null);

    const refreshQuiltVersions = useCallback(async () =>
    {
        setQuiltVersions(await getQuiltVersions());
    }, []);

    useEffect(() =>
    {
        refreshQuiltVersions()
            .then(() => console.log("Loaded quilt versions successfully."))
            .catch(error =>
            {
                console.error("Failed to load quilt versions:", error);
                setQuiltVersions(null); // Reset to null on error
                addToast({
                    title: "Error",
                    description: "Failed to load quilt versions. Please try again later.",
                    color: "danger"
                });
            });
    }, []);

    return (
        <QuiltVersionsContext.Provider value={{quiltVersions, refreshQuiltVersions}}>
            {children}
        </QuiltVersionsContext.Provider>
    );
}

export function useQuiltVersions(): QuiltVersionsContextType
{
    const context = useContext(QuiltVersionsContext);
    if (!context)
    {
        throw new Error("useQuiltVersions must be used within a QuiltVersionsProvider");
    }
    return context;
}