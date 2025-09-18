import {useCallback, useEffect, useRef, useState} from "react";
import {ModItemSkeleton} from "../ModItem.tsx";
import {InstalledMod, useServer} from "../../../../../providers/ServerProvider.tsx";
import {InstalledModItem} from "./InstalledModItem.tsx";

type ModListProps = {
    searchQuery: string;
    limit: number;
    offset: number;
    loaders: Loader[];
}

type Loader = "any" | "fabric" | "forge" | "quilt" | "neoforge";

export function InstalledModList(props: ModListProps)
{
    const {searchQuery, limit, offset} = props;
    const {getInstalledMods, server} = useServer();
    const [allMods, setAllMods] = useState<InstalledMod[]>([]);
    const [filteredMods, setFilteredMods] = useState<InstalledMod[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    // Keep a ref to the latest list so comparisons are correct inside callbacks
    const allModsRef = useRef<InstalledMod[]>([]);
    useEffect(() => { allModsRef.current = allMods; }, [allMods]);

    // Load mods from server
    useEffect(() =>
    {
        let isMounted = true;
        setIsLoading(true);
        getInstalledMods()
            .then(mods => {
                if (!isMounted) return;
                setAllMods(mods);
            })
            .catch((error) =>
            {
                console.error("Failed to load installed mods:", error);
            })
            .finally(() => isMounted && setIsLoading(false));
        return () => { isMounted = false; };
    }, [getInstalledMods]);

    useEffect(() =>
    {
        refreshMods();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [server]);

    const shallowEqual = (a: InstalledMod[], b: InstalledMod[]) =>
    {
        if (a === b) return true;
        if (a.length !== b.length) return false;
        // Cheap compare by stable fields; adjust if needed
        for (let i = 0; i < a.length; i++)
        {
            if (a[i].mod_id !== b[i].mod_id) return false;
        }
        return true;
    };

    const refreshMods = useCallback(async () =>
    {
        const mods = await getInstalledMods();
        // Only update state if actually changed
        if (!shallowEqual(allModsRef.current, mods))
        {
            setIsLoading(true);
            setAllMods(mods);
            // Small delay for skeletons if desired; otherwise remove
            setTimeout(() => setIsLoading(false), 100);
        }
    }, [getInstalledMods]);

    // Filter mods based on search query
    useEffect(() =>
    {
        if (!searchQuery.trim())
        {
            setFilteredMods(allMods);
        } else
        {
            const query = searchQuery.toLowerCase();
            const filtered = allMods.filter(mod =>
                mod.name.toLowerCase().includes(query) ||
                mod.description.toLowerCase().includes(query) ||
                mod.authors.some(author => author.toLowerCase().includes(query)) ||
                mod.mod_id.toLowerCase().includes(query)
            );
            setFilteredMods(filtered);
        }
    }, [searchQuery, allMods]);

    // Apply pagination
    const paginatedMods = filteredMods.slice(offset, offset + limit);

    return (
        <>
            {isLoading ?
                Array.from({length: 10}).map((_, index) => <ModItemSkeleton key={index}/>) :
                paginatedMods.map((mod) => <InstalledModItem key={mod.mod_id} {...mod}/>)
            }
            {!isLoading && paginatedMods.length === 0 && (
                <div className="text-center text-default-500 py-8">
                    {searchQuery ? "No mods found matching your search." : "No mods installed."}
                </div>
            )}
        </>
    );
}