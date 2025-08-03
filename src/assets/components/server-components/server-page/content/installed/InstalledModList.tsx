import {useCallback, useEffect, useState} from "react";
import {ModItemSkeleton} from "../ModItem.tsx";
import {InstalledMod, useServer} from "../../../../../providers/ServerProvider.tsx";
import {InstalledModItem} from "./InstalledModItem.tsx";

type ModListProps = {
    searchQuery: string;
    limit: number;
    offset: number;
}

export function InstalledModList(props: ModListProps)
{
    const {searchQuery, limit, offset} = props;
    const {getInstalledMods, server} = useServer();
    const [allMods, setAllMods] = useState<InstalledMod[]>([]);
    const [filteredMods, setFilteredMods] = useState<InstalledMod[]>([]);
    const [isLoading, setIsLoading] = useState(false);


    // Load mods from server
    useEffect(() =>
    {
        setIsLoading(true);
        getInstalledMods()
            .then(setAllMods)
            .catch((error) =>
            {
                console.error("Failed to load installed mods:", error);
            })
            .finally(() => setIsLoading(false));
    }, []); // Only load once when component mounts

    useEffect(() =>
    {
        refreshMods();
    }, [server]);

    const refreshMods = useCallback(async () =>
    {
        const mods = await getInstalledMods();
        if (JSON.stringify(allMods) != JSON.stringify(mods))
        {
            console.log("Refreshing installed mods:", mods, allMods);
            setIsLoading(true);
            setAllMods(mods);
            setTimeout(() => setIsLoading(false), 100);
        }
    }, [server]);

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