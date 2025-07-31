import {useEffect, useRef, useState} from "react";
import CurseForge from "../../../../../ts/curseforge_api.ts";
import {ModListProps} from "../ServerContent.tsx";
import {ModItem, ModItemProps, ModItemSkeleton} from "../ModItem.tsx";

export function ModList(props: ModListProps)
{
    const {searchQuery, minecraftVersion, loader, category, limit, offset} = props;
    const [mods, setMods] = useState<ModItemProps[]>([]);
    const curseForge = useRef(CurseForge.getWithDefaultAPI());
    const abortSignal = useRef(new AbortController());
    const [isLoading, setIsLoading] = useState(false);


    useEffect(() =>
    {
        setIsLoading(true);
        // Abort any previous request
        if (abortSignal.current) abortSignal.current.abort();
        abortSignal.current = new AbortController();
        curseForge.current.search({
            category: category[0],
            limit: limit,
            offset: offset,
            loader: loader[0],
            minecraftVersion: minecraftVersion[0],
            query: searchQuery
        }, abortSignal.current.signal).then(results =>
        {
            console.log("CurseForge Search Results:", results);
            if (results.data && results.data.length > 0)
            {
                setMods(results.data.map(mod => ({
                    modId: mod.id.toString(),
                    platform: "curseforge",
                    description: mod.summary,
                    iconUrl: mod.logo ? mod.logo.url : undefined,
                    name: mod.name,
                    downloadCount: mod.downloadCount,
                    author: mod.authors[0].name,
                    categories: mod.categories.map(category => category.name),
                    lastUpdated: new Date(mod.dateModified),
                    slug: mod.slug
                } as ModItemProps)));
            } else
            {
                setMods([]);
                console.warn("No mods found for the given search criteria.");
            }
        }).catch((e: any | Error) =>
            {
                if (e.name === "AbortError")
                {
                    console.log("CurseForge search request aborted");
                } else
                {
                    console.error("Error fetching CurseForge mods:", e);
                }
            }
        ).finally(() => setIsLoading(false));
    }, [searchQuery, minecraftVersion, loader, category, limit, offset]);

    if (!curseForge.current)
    {
        return <div>Error loading CurseForge API</div>;
    }
    return (
        <>
            {isLoading ? Array.from({length: 10}).map(() => <ModItemSkeleton/>) :
                mods.map(mod => <ModItem {...mod}/>)}
        </>
    );
}