import {Button, Input, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {useEffect, useState} from "react";
import {ContentFilters as ModrinthContentFilters} from "./modrinth/ContentFilters.tsx";
import {ContentFilters as CurseForgeContentFilters} from "./curseforge/ContentFilters.tsx";
import {ModList as ModrinthModList} from "./modrinth/ModList.tsx";
import {ModList as CurseForgeModList} from "./curseforge/ModList.tsx";
import {useSearchParams} from "react-router-dom";

export type ModListProps = {
    searchQuery: string;
    minecraftVersion: string[];
    loader: string[];
    category: string[];
    limit: number;
    offset: number;
}

export function ServerContent()
{
    const [selectedPlatform, setSelectedPlatform] = useState("modrinth");
    const [search, setSearch] = useState("");
    const [loaders, setLoaders] = useState<string[]>([]);
    const [minecraftVersions, setMinecraftVersions] = useState<string[]>([]);
    const [categories, setCategories] = useState<string[]>([]);
    const [queryParams, setQueryParams] = useSearchParams();

    // On mount set the initial state from the url query parameters
    useEffect(() =>
    {
        const platform = queryParams.get("platform");
        const searchQuery = queryParams.get("search") || "";
        const loadersQuery = queryParams.get("loaders") || "";
        const minecraftVersionsQuery = queryParams.get("minecraftVersions") || "";
        const categoriesQuery = queryParams.get("categories") || "";

        if (platform && (platform === "modrinth" || platform === "curseforge"))
        {
            setSelectedPlatform(platform);
        }

        setSearch(searchQuery);
        setLoaders(loadersQuery ? loadersQuery.split(",").filter(Boolean) : []);
        setMinecraftVersions(minecraftVersionsQuery ? minecraftVersionsQuery.split(",").filter(Boolean) : []);
        setCategories(categoriesQuery ? categoriesQuery.split(",").filter(Boolean) : []);
    }, []);

    // Set the url query parameters when the content tab is selected
    useEffect(() =>
    {
        if (queryParams.get("tab") !== "content") return;

        const newParams = new URLSearchParams(queryParams);
        newParams.set("platform", selectedPlatform);
        newParams.set("search", search);
        newParams.set("loaders", loaders.join(","));
        newParams.set("minecraftVersions", minecraftVersions.join(","));
        newParams.set("categories", categories.join(","));

        setQueryParams(newParams);
    }, [search, loaders, minecraftVersions, categories, selectedPlatform, queryParams, setQueryParams]);

    return (
        <div className={"flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative"}>

            {/* Top Bar */}
            <div className={"flex flex-row gap-4 items-center justify-between z-20"}>
                <Input
                    label={"Search"}
                    radius={"none"}
                    className={"font-minecraft-body"}
                    placeholder={"Ex: AE2, Applied Energistics 2, Ender IO, etc."}
                    autoComplete={"off"}
                    autoCorrect={"off"}
                    startContent={<Icon icon={"pixelarticons:search"}/>}
                    size={"sm"}
                    value={search}
                    onValueChange={setSearch}
                    endContent={
                        <Tooltip content={"Submit!"}>
                            <Button isIconOnly radius={"none"} variant={"light"}>
                                <Icon icon={"pixelarticons:arrow-right"}/>
                            </Button>
                        </Tooltip>
                    }
                />
                <Tabs
                    size={"lg"}
                    radius={"none"}
                    classNames={{
                        cursor: selectedPlatform === "modrinth" ? "!bg-[#1bd96a]" : selectedPlatform === "curseforge" ? "!bg-[#f16436]" : ""
                    }}
                    selectedKey={selectedPlatform}
                    onSelectionChange={value => setSelectedPlatform(value as string)}
                >
                    <Tab key={"modrinth"} title={
                        <Tooltip content={"Modrinth"}>
                            <Icon icon={"simple-icons:modrinth"} className={selectedPlatform === "modrinth" ? "text-black" : ""}/>
                        </Tooltip>
                    }/>
                    <Tab key={"curseforge"} title={
                        <Tooltip content={"CurseForge"}>
                            <Icon icon={"simple-icons:curseforge"}/>
                        </Tooltip>
                    }/>
                </Tabs>
            </div>

            <div className={"flex flex-row gap-4 h-full relative max-h-[calc(100dvh_-_490px)] min-h-[220px] z-20"}>
                {/* Side Panel */}
                <div className={"bg-default-100 min-w-64 w-64  max-w-64  h-full overflow-y-auto flex flex-col gap-2 p-2 pr-6"}>
                    {selectedPlatform === "modrinth" ?
                        <ModrinthContentFilters
                            selectedCategories={categories}
                            onCategoryChange={setCategories}
                            selectedLoaders={loaders}
                            onLoaderChange={setLoaders}
                            selectedGameVersions={minecraftVersions}
                            onGameVersionChange={setMinecraftVersions}
                        />
                        : selectedPlatform === "curseforge" ?
                            <CurseForgeContentFilters
                                selectedCategories={categories}
                                onCategoryChange={setCategories}
                                selectedLoaders={loaders}
                                onLoaderChange={setLoaders}
                                selectedGameVersions={minecraftVersions}
                                onGameVersionChange={setMinecraftVersions}
                            />
                            : null
                    }
                </div>
                <div className={"h-full overflow-y-auto flex flex-col gap-2 p-2 pr-6 grow"}>
                    {selectedPlatform === "modrinth" ?
                        <ModrinthModList
                            searchQuery={search}
                            loader={loaders}
                            minecraftVersion={minecraftVersions}
                            category={categories}
                            limit={20}
                            offset={0}
                        />
                        : selectedPlatform === "curseforge" ?
                            <CurseForgeModList
                                searchQuery={search}
                                loader={loaders}
                                minecraftVersion={minecraftVersions}
                                category={categories}
                                limit={20}
                                offset={0}
                            />
                            : null
                    }
                </div>

            </div>

            <span className={"z-10 absolute inset-0 data-[selected=modrinth]:bg-[#1bd96a] data-[selected=curseforge]:bg-[#f16436] opacity-5 transition-background duration-200"} data-selected={selectedPlatform}/>
        </div>
    );
}