import {useNavigate, useParams, useSearchParams} from "react-router-dom";
import {useEffect, useState, useRef} from "react";
import {Button, Input, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {motion, AnimatePresence} from "framer-motion";
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

// Platform data structure for lazy loading
type PlatformData = {
    modpacks: ModpackItemProps[];
    isLoading: boolean;
    isLoaded: boolean;
};

const PLATFORMS: ModpackPlatform[] = ["modrinth", "curseforge", "atlauncher", "technic"];

export default function DiscoverPage()
{
    const {type, platform} = useParams();
    const navigate = useNavigate();
    const [searchParams, setSearchParams] = useSearchParams();
    const prevPlatformRef = useRef<ModpackPlatform | null>(null);

    // Initialize state from URL params directly to avoid double-loading
    const [selectedPlatform, setSelectedPlatform] = useState<ModpackPlatform>(() =>
    {
        if (platform && PLATFORMS.includes(platform as ModpackPlatform))
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

    // Store data per platform for lazy loading and persistence
    const [platformData, setPlatformData] = useState<Record<ModpackPlatform, PlatformData>>({
        modrinth: {modpacks: [], isLoading: false, isLoaded: false},
        curseforge: {modpacks: [], isLoading: false, isLoaded: false},
        atlauncher: {modpacks: [], isLoading: false, isLoaded: false},
        technic: {modpacks: [], isLoading: false, isLoaded: false}
    });

    // Track filter version to invalidate cache when filters change
    const [filterVersion, setFilterVersion] = useState(0);

    // Animation direction state
    const [direction, setDirection] = useState<number>(0);

    // Update platform when URL changes and calculate animation direction
    useEffect(() =>
    {
        if (platform && PLATFORMS.includes(platform as ModpackPlatform))
        {
            const newPlatform = platform as ModpackPlatform;
            if (prevPlatformRef.current)
            {
                const prevIndex = PLATFORMS.indexOf(prevPlatformRef.current);
                const newIndex = PLATFORMS.indexOf(newPlatform);
                setDirection(newIndex - prevIndex);
            }
            prevPlatformRef.current = newPlatform;
            setSelectedPlatform(newPlatform);
        } else if (!platform)
        {
            // Default to modrinth if no platform specified
            navigate(`/app/discover/${type}/modrinth`, {replace: true});
        }
    }, [platform, type, navigate]);

    // Update URL when filters change and increment filter version to invalidate cache
    useEffect(() =>
    {
        const newParams = new URLSearchParams();
        if (search) newParams.set("search", search);
        if (minecraftVersions.length > 0) newParams.set("minecraftVersions", minecraftVersions.join(","));
        if (categories.length > 0) newParams.set("categories", categories.join(","));

        setSearchParams(newParams);

        // Reset isLoaded for all platforms to force refetch with new filters
        setPlatformData(prev => {
            const updated: Record<ModpackPlatform, PlatformData> = {...prev};
            for (const platform of PLATFORMS) {
                updated[platform] = {...updated[platform], isLoaded: false};
            }
            return updated;
        });

        // Increment filter version to trigger refetch
        setFilterVersion(prev => prev + 1);
    }, [search, minecraftVersions, categories, setSearchParams]);

    // Fetch modpacks when platform or filters change (with lazy loading)
    useEffect(() =>
    {
        if (!selectedPlatform) return;

        const currentData = platformData[selectedPlatform];

        // Skip if already loaded (lazy loading) or already loading
        if (currentData.isLoaded || currentData.isLoading) return;

        const fetchModpacks = async () =>
        {
            // Set loading state for current platform
            setPlatformData(prev => ({
                ...prev,
                [selectedPlatform]: {...prev[selectedPlatform], isLoading: true}
            }));

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

                setPlatformData(prev => ({
                    ...prev,
                    [selectedPlatform]: {
                        modpacks: results,
                        isLoading: false,
                        isLoaded: true
                    }
                }));
            } catch (error)
            {
                console.error(`Failed to fetch ${selectedPlatform} modpacks:`, error);
                setPlatformData(prev => ({
                    ...prev,
                    [selectedPlatform]: {
                        modpacks: [],
                        isLoading: false,
                        isLoaded: true
                    }
                }));
            }
        };

        fetchModpacks();
    }, [selectedPlatform, filterVersion]);

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

                {/* Main Content - Modpack List with Carousel Animation */}
                <div className={"h-full overflow-hidden flex flex-col gap-2 p-2 pr-6 grow relative"}>
                    <AnimatePresence initial={false} custom={direction} mode="wait">
                        <motion.div
                            key={selectedPlatform}
                            custom={direction}
                            variants={{
                                enter: (direction: number) => ({
                                    x: direction > 0 ? 1000 : -1000,
                                    opacity: 0
                                }),
                                center: {
                                    x: 0,
                                    opacity: 1
                                },
                                exit: (direction: number) => ({
                                    x: direction > 0 ? -1000 : 1000,
                                    opacity: 0
                                })
                            }}
                            initial="enter"
                            animate="center"
                            exit="exit"
                            transition={{
                                x: {type: "spring", stiffness: 300, damping: 30},
                                opacity: {duration: 0.2}
                            }}
                            className={"h-full overflow-y-auto flex flex-col gap-2 absolute inset-0"}
                        >
                            {platformData[selectedPlatform].isLoading ? (
                                Array.from({length: 10}).map((_, index) => <ModpackItemSkeleton key={index}/>)
                            ) : platformData[selectedPlatform].modpacks.length === 0 ? (
                                <div className={"text-center text-default-500 mt-8"}>
                                    <Icon icon={"pixelarticons:folder-open"} width={48} height={48} className={"mx-auto mb-2"}/>
                                    <p>No modpacks found</p>
                                    <p className={"text-sm"}>Try adjusting your search or filters</p>
                                </div>
                            ) : (
                                platformData[selectedPlatform].modpacks.map((modpack, index) => (
                                    <ModpackItem key={modpack.packId || index} {...modpack}/>
                                ))
                            )}
                        </motion.div>
                    </AnimatePresence>
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
