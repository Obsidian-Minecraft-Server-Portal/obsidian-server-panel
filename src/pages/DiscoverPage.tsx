import {useNavigate, useParams, useSearchParams} from "react-router-dom";
import {useEffect, useState} from "react";
import {Button, Input, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../components/extended/Tooltip.tsx";
import {ModpackFilters} from "../components/discover/ModpackFilters.tsx";
import {ModpackItem, ModpackItemSkeleton} from "../components/discover/ModpackItem.tsx";
import type {ModpackItemProps, ModpackPlatform} from "../types/ModpackTypes.ts";
import {
    searchModrinthModpacks,
    searchCurseForgeModpacks,
    searchATLauncherModpacks,
    searchTechnicModpacks
} from "../utils/modpack-api.ts";

export default function DiscoverPage()
{
    const {type, platform} = useParams();
    const navigate = useNavigate();
    const [searchParams, setSearchParams] = useSearchParams();

    // Initialize state from URL params directly to avoid double-loading
    const [selectedPlatform, setSelectedPlatform] = useState<ModpackPlatform>(() =>
    {
        if (platform && ["modrinth", "curseforge", "atlauncher", "technic"].includes(platform))
        {
            return platform as ModpackPlatform;
        }
        return "modrinth";
    });
    const [search, setSearch] = useState(searchParams.get("search") || "");
    const [minecraftVersions, setMinecraftVersions] = useState<string[]>(() =>
    {
        const param = searchParams.get("minecraftVersions");
        return param ? param.split(",").filter(Boolean) : [];
    });
    const [categories, setCategories] = useState<string[]>(() =>
    {
        const param = searchParams.get("categories");
        return param ? param.split(",").filter(Boolean) : [];
    });
    const [modpacks, setModpacks] = useState<ModpackItemProps[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    // Update platform when URL changes
    useEffect(() =>
    {
        if (platform && ["modrinth", "curseforge", "atlauncher", "technic"].includes(platform))
        {
            setSelectedPlatform(platform as ModpackPlatform);
        } else if (!platform)
        {
            // Default to modrinth if no platform specified
            navigate(`/app/discover/${type}/modrinth`, {replace: true});
        }
    }, [platform, type, navigate]);

    // Update URL when filters change
    useEffect(() =>
    {
        const newParams = new URLSearchParams();
        if (search) newParams.set("search", search);
        if (minecraftVersions.length > 0) newParams.set("minecraftVersions", minecraftVersions.join(","));
        if (categories.length > 0) newParams.set("categories", categories.join(","));

        setSearchParams(newParams);
    }, [search, minecraftVersions, categories, setSearchParams]);

    // Fetch modpacks when platform or filters change
    useEffect(() =>
    {
        if (!selectedPlatform) return;

        const fetchModpacks = async () =>
        {
            setIsLoading(true);
            try
            {
                let results: ModpackItemProps[] = [];

                switch (selectedPlatform)
                {
                    case "modrinth":
                        {
                            const facets: any[] = [];
                            if (minecraftVersions.length > 0)
                            {
                                facets.push(minecraftVersions.map(v => `versions:${v}`));
                            }
                            if (categories.length > 0)
                            {
                                facets.push(categories.map(c => `categories:${c}`));
                            }
                            results = await searchModrinthModpacks({
                                query: search,
                                facets: facets.length > 0 ? JSON.stringify(facets) : undefined,
                                limit: 20,
                                offset: 0
                            });
                        }
                        break;

                    case "curseforge":
                        results = await searchCurseForgeModpacks({
                            query: search,
                            gameVersion: minecraftVersions[0], // CurseForge only supports one version at a time
                            limit: 20,
                            offset: 0
                        });
                        break;

                    case "atlauncher":
                        results = await searchATLauncherModpacks({
                            query: search
                        });
                        // Client-side filtering for ATLauncher
                        if (minecraftVersions.length > 0 || categories.length > 0)
                        {
                            results = results.filter(pack =>
                            {
                                const versionMatch = minecraftVersions.length === 0 || minecraftVersions.some(v => pack.categories.includes(v));
                                const categoryMatch = categories.length === 0 || categories.some(c => pack.categories.includes(c));
                                return versionMatch && categoryMatch;
                            });
                        }
                        break;

                    case "technic":
                        results = await searchTechnicModpacks({
                            query: search,
                            limit: 20,
                            offset: 0
                        });
                        break;
                }

                setModpacks(results);
            } catch (error)
            {
                console.error(`Failed to fetch ${selectedPlatform} modpacks:`, error);
                setModpacks([]);
            } finally
            {
                setIsLoading(false);
            }
        };

        fetchModpacks();
    }, [selectedPlatform, search, minecraftVersions, categories]);

    // Update URL when platform changes
    useEffect(() =>
    {
        if (selectedPlatform && type)
        {
            navigate(`/app/discover/${type}/${selectedPlatform}${location.search}`, {replace: true});
        }
    }, [selectedPlatform, type, navigate]);

    const getPlatformColor = (plat: ModpackPlatform) =>
    {
        switch (plat)
        {
            case "modrinth":
                return "!bg-[#1bd96a]";
            case "curseforge":
                return "!bg-[#f16436]";
            case "atlauncher":
                return "!bg-[#89c236]";
            case "technic":
                return "!bg-[#e74c3c]";
            default:
                return "";
        }
    };

    const getPlatformIcon = (plat: ModpackPlatform): {type: "icon" | "image"; value: string} =>
    {
        switch (plat)
        {
            case "modrinth":
                return {type: "icon", value: "simple-icons:modrinth"};
            case "curseforge":
                return {type: "icon", value: "simple-icons:curseforge"};
            case "atlauncher":
                return {type: "image", value: "https://atlauncher.com/assets/images/logo.svg"};
            case "technic":
                return {type: "icon", value: "pixelarticons:zap"};
            default:
                return {type: "icon", value: "pixelarticons:folder"};
        }
    };

    return (
        <div className={"flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_100px)] h-screen min-h-[300px] relative"}>
            {/* Top Bar */}
            <div className={"flex flex-row gap-4 items-center justify-between z-20"}>
                <Input
                    label={"Search Modpacks"}
                    radius={"none"}
                    className={"font-minecraft-body"}
                    placeholder={"Ex: All the Mods, FTB, RLCraft, etc."}
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
                        cursor: getPlatformColor(selectedPlatform),
                        tabContent: "w-[20px]"
                    }}
                    selectedKey={selectedPlatform}
                    onSelectionChange={value => setSelectedPlatform(value as ModpackPlatform)}
                >
                    <Tab
                        key={"modrinth"}
                        title={
                            <Tooltip content={"Modrinth"}>
                                <Icon
                                    icon={getPlatformIcon("modrinth").value}
                                    className={selectedPlatform === "modrinth" ? "text-black" : ""}
                                />
                            </Tooltip>
                        }
                    />
                    <Tab
                        key={"curseforge"}
                        title={
                            <Tooltip content={"CurseForge"}>
                                <Icon icon={getPlatformIcon("curseforge").value}/>
                            </Tooltip>
                        }
                    />
                    <Tab
                        key={"atlauncher"}
                        title={
                            <Tooltip content={"ATLauncher"}>
                                <img
                                    src={getPlatformIcon("atlauncher").value}
                                    alt="ATLauncher"
                                    className="w-5 h-5"
                                />
                            </Tooltip>
                        }
                    />
                    <Tab
                        key={"technic"}
                        title={
                            <Tooltip content={"Technic"}>
                                <Icon icon={getPlatformIcon("technic").value}/>
                            </Tooltip>
                        }
                    />
                </Tabs>
            </div>

            <div className={"flex flex-row gap-4 h-full relative max-h-[calc(100dvh_-_170px)] min-h-[220px] z-20"}>
                {/* Side Panel - Filters */}
                <div className={"bg-default-100 min-w-64 w-64 max-w-64 h-full overflow-y-auto flex flex-col gap-2 p-2 pr-6"}>
                    <ModpackFilters
                        selectedGameVersions={minecraftVersions}
                        selectedCategories={categories}
                        onGameVersionChange={setMinecraftVersions}
                        onCategoryChange={setCategories}
                        platform={selectedPlatform}
                    />
                </div>

                {/* Main Content - Modpack List */}
                <div className={"h-full overflow-y-auto flex flex-col gap-2 p-2 pr-6 grow"}>
                    {isLoading ? (
                        Array.from({length: 10}).map((_, index) => <ModpackItemSkeleton key={index}/>)
                    ) : modpacks.length === 0 ? (
                        <div className={"text-center text-default-500 mt-8"}>
                            <Icon icon={"pixelarticons:folder-open"} width={48} height={48} className={"mx-auto mb-2"}/>
                            <p>No modpacks found</p>
                            <p className={"text-sm"}>Try adjusting your search or filters</p>
                        </div>
                    ) : (
                        modpacks.map((modpack, index) => (
                            <ModpackItem key={modpack.packId || index} {...modpack}/>
                        ))
                    )}
                </div>
            </div>

            {/* Background color overlay based on selected platform */}
            <span
                className={"z-10 absolute inset-0 opacity-5 transition-background duration-200"}
                style={{
                    backgroundColor:
                        selectedPlatform === "modrinth" ? "#1bd96a" :
                            selectedPlatform === "curseforge" ? "#f16436" :
                                selectedPlatform === "atlauncher" ? "#89c236" :
                                    selectedPlatform === "technic" ? "#e74c3c" :
                                        "transparent"
                }}
            />
        </div>
    );
}
