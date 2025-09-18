import {useEffect, useRef, useState} from "react";
import {ModListProps} from "../ServerContent.tsx";
import {ModItem, ModItemProps, ModItemSkeleton} from "../ModItem.tsx";
import Modrinth, {ModrinthSearchResult} from "../../../../../ts/modrinth_api.ts";

export function ModList(props: ModListProps)
{
    const {searchQuery, minecraftVersion, loader, category, limit, offset} = props;
    const [mods, setMods] = useState<ModItemProps[]>([]);
    const abortSignal = useRef(new AbortController());
    const [isLoading, setIsLoading] = useState(false);
    const modrinth = useRef(Modrinth.getInstance());

    useEffect(() =>
    {
        setIsLoading(true);
        // Abort any previous request
        if (abortSignal.current) abortSignal.current.abort();
        abortSignal.current = new AbortController();

        // Build facets for Modrinth search
        const facets = Modrinth.buildFacets(category, minecraftVersion, loader);

        modrinth.current.search({
            query: searchQuery || undefined,
            facets: facets.length > 0 ? facets : undefined,
            limit: limit,
            offset: offset,
            index: "relevance"
        }, abortSignal.current.signal)
            .then((results: ModrinthSearchResult) =>
            {
                console.log("Modrinth Search Results:", results);
                if (results.hits && results.hits.length > 0)
                {
                    setMods(results.hits.map(project => ({
                        modId: project.project_id,
                        platform: "modrinth",
                        description: project.description,
                        iconUrl: project.icon_url,
                        name: project.title,
                        downloadCount: project.downloads,
                        author: project.author,
                        categories: project.categories,
                        lastUpdated: new Date(project.date_modified)
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
                console.log("Modrinth search request aborted");
            } else
            {
                console.error("Error fetching Modrinth mods:", e);
            }
        }).finally(() => setIsLoading(false));
    }, [searchQuery, minecraftVersion, loader, category, limit, offset]);

    return (
        <>
            {isLoading ? Array.from({length: 10}).map((_, index) => <ModItemSkeleton key={index}/>) :
                mods.map((mod, index) => <ModItem key={mod.modId || index} {...mod}/>)}
        </>
    );
}