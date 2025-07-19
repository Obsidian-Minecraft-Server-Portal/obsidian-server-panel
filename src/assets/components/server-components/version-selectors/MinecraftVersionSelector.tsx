import {Autocomplete, AutocompleteItem, Button} from "@heroui/react";
import {Tooltip} from "../../extended/Tooltip.tsx";
import {Icon} from "@iconify-icon/react";
import {useEffect, useState} from "react";
import {useMinecraftVersions} from "../../../providers/MinecraftVersionsProvider.tsx";

type MinecraftVersionSelectorProps = {
    onVersionChange?: (version: string | undefined) => void;
    version?: string | undefined;
}

export function MinecraftVersionSelector(props: MinecraftVersionSelectorProps)
{
    const {minecraftVersions} = useMinecraftVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(props.version);
    const [versions, setVersions] = useState<string[]>([]);
    const [showSnapshots, setShowSnapshots] = useState(false);
    const [showOlderVersions, setShowOlderVersions] = useState(false);
    useEffect(() =>
    {
        if (!minecraftVersions) return;

        setVersions(minecraftVersions.versions.filter(i => i.type === "release" || (showSnapshots && i.type === "snapshot") || (showOlderVersions && (i.type === "old_beta" || i.type === "old_alpha"))).map(version => version.id));
        setSelectedVersion(showSnapshots ? minecraftVersions.latest.snapshot : minecraftVersions.latest.release);
    }, [showSnapshots, showOlderVersions, minecraftVersions]);

    useEffect(() =>
    {
        if (props.onVersionChange) props.onVersionChange(selectedVersion);
    }, [selectedVersion, showOlderVersions, showSnapshots]);

    return (
        <div className={"flex flex-row gap-2 items-center"}>
            <Autocomplete
                label={`Minecraft Version`}
                radius={"none"}
                className={"font-minecraft-body"}
                classNames={{
                    base: "capitalize",
                    popoverContent: "rounded-none border-primary border-1"
                }}
                size={"sm"}
                selectedKey={selectedVersion}
                onSelectionChange={value => setSelectedVersion(value as string)}
                showScrollIndicators
            >
                {versions.map((version) => (
                    <AutocompleteItem
                        key={version}
                        className={"font-minecraft-body"}
                        textValue={version}
                    >
                        {version}
                    </AutocompleteItem>
                ))}
            </Autocomplete>
            <Tooltip content={"Show snapshots"}>
                <Button isIconOnly radius={"none"} size={"lg"} color={showSnapshots ? "primary" : "default"} onPress={() => setShowSnapshots(prev => !prev)}>
                    <Icon icon={"pixelarticons:bug"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Show Older Versions"}>
                <Button isIconOnly radius={"none"} size={"lg"} color={showOlderVersions ? "primary" : "default"} onPress={() => setShowOlderVersions(prev => !prev)}>
                    <Icon icon={"pixelarticons:archive"}/>
                </Button>
            </Tooltip>
        </div>
    );
}