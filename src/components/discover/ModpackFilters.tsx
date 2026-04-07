import {ListBox, ListBoxItem, ScrollShadow} from "@heroui/react";
import {useMinecraftVersions} from "../../providers/LoaderVersionProviders/MinecraftVersionsProvider.tsx";

type ModpackFiltersProps = {
    selectedGameVersions: string[];
    selectedCategories: string[];
    onGameVersionChange: (versions: string[]) => void;
    onCategoryChange: (categories: string[]) => void;
    platform: "modrinth" | "curseforge" | "atlauncher" | "technic";
}

export function ModpackFilters(props: ModpackFiltersProps)
{
    const {selectedGameVersions, selectedCategories, onGameVersionChange, onCategoryChange, platform} = props;
    const {minecraftVersions} = useMinecraftVersions();

    // Categories differ slightly by platform
    const getCategories = () =>
    {
        switch (platform)
        {
            case "modrinth":
            case "curseforge":
                return [
                    "Adventure and RPG",
                    "Challenging",
                    "Combat / PvP",
                    "Exploration",
                    "Extra Large",
                    "Hardcore",
                    "Kitchen Sink",
                    "Large",
                    "Lightweight",
                    "Magic",
                    "Medium",
                    "Multiplayer",
                    "Optimization",
                    "Quests",
                    "Sci-Fi",
                    "Small / Light",
                    "Tech",
                    "Technology",
                    "Vanilla+"
                ];
            case "atlauncher":
                return [
                    "Adventure",
                    "Magic",
                    "Tech",
                    "Quests",
                    "Hardcore",
                    "Lightweight",
                    "PvP"
                ];
            case "technic":
                return [
                    "Adventure",
                    "Magic",
                    "Tech",
                    "Survival",
                    "Creative"
                ];
            default:
                return [];
        }
    };

    const categories = getCategories();

    const releaseVersions = minecraftVersions?.versions
        ?.filter((v) => v.type === "release")
        .map((v) => v.id)
        .slice(0, 20) || [];

    return (
        <>
            {/* Minecraft Version Filter */}
            <div className={"flex flex-col gap-2"}>
                <label className={"font-minecraft-body text-large"}>Minecraft Version</label>
                <ScrollShadow className={"max-h-[200px]"}>
                    <ListBox
                        selectionMode={"multiple"}
                        selectedKeys={selectedGameVersions}
                        onSelectionChange={(keys: any) => onGameVersionChange([...keys] as string[])}
                        className="rounded-none font-minecraft-body"
                    >
                        {releaseVersions.map((version: string) => (
                            <ListBoxItem key={version}>
                                {version}
                            </ListBoxItem>
                        ))}
                    </ListBox>
                </ScrollShadow>
            </div>

            {/* Categories Filter */}
            {categories.length > 0 && (
                <div className={"flex flex-col gap-2"}>
                    <label className={"font-minecraft-body text-large"}>Categories</label>
                    <ScrollShadow className={"max-h-[200px]"}>
                        <ListBox
                            selectionMode={"multiple"}
                            selectedKeys={selectedCategories}
                            onSelectionChange={(keys: any) => onCategoryChange([...keys] as string[])}
                            className="rounded-none font-minecraft-body"
                        >
                            {categories.map((category: string) => (
                                <ListBoxItem key={category}>
                                    {category}
                                </ListBoxItem>
                            ))}
                        </ListBox>
                    </ScrollShadow>
                </div>
            )}
        </>
    );
}
