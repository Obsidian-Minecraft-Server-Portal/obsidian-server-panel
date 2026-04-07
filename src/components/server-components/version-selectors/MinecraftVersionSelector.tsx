import {Autocomplete, ListBoxItem, Button} from "@heroui/react";
import {Tooltip} from "../../extended/Tooltip.tsx";
import {Icon} from "@iconify-icon/react";
import {useEffect, useState} from "react";
import {useMinecraftVersions} from "../../../providers/LoaderVersionProviders/MinecraftVersionsProvider.tsx";

type MinecraftVersionSelectorProps = {
    onVersionChange?: (version: string | undefined, url: string | undefined) => void;
    version?: string | undefined;
    isDisabled?: boolean;
}

export function MinecraftVersionSelector(props: MinecraftVersionSelectorProps)
{
    const {onVersionChange, version, isDisabled} = props;
    const {minecraftVersions} = useMinecraftVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(version);
    const [versions, setVersions] = useState<string[]>([]);
    const [showSnapshots, setShowSnapshots] = useState(false);
    const [showOlderVersions, setShowOlderVersions] = useState(false);
    useEffect(() =>
    {
        if (!minecraftVersions) return;

        setVersions(minecraftVersions.versions.filter(i => i.type === "release" || (showSnapshots && i.type === "snapshot") || (showOlderVersions && (i.type === "old_beta" || i.type === "old_alpha"))).map(version => version.id));

        // Only set default version if no version is controlled from parent
        if (!version && !selectedVersion) {
            setSelectedVersion(showSnapshots ? minecraftVersions.latest.snapshot : minecraftVersions.latest.release);
        }
    }, [showSnapshots, showOlderVersions, minecraftVersions]); // Removed version and selectedVersion from deps

    useEffect(() =>
    {
        if (onVersionChange)
        {
            const url = minecraftVersions?.versions.find(i => i.id === selectedVersion)?.url;
            onVersionChange(selectedVersion, url);
        }
    }, [selectedVersion, minecraftVersions, onVersionChange]); // Removed showOlderVersions and showSnapshots from deps

    useEffect(() =>
    {
        if (version !== undefined) {
            setSelectedVersion(version);
        }
    }, [version]);

    return (
        <div className={"flex flex-row gap-2 items-center"}>
            <Autocomplete
                className={"font-minecraft-body rounded-none"}
                selectedKey={selectedVersion}
                onSelectionChange={value => setSelectedVersion(value as string)}
                isDisabled={isDisabled}
            >
                {versions.map((version) => (
                    <ListBoxItem
                        key={version}
                        className={"font-minecraft-body"}
                        textValue={version}
                    >
                        {version}
                    </ListBoxItem>
                ))}
            </Autocomplete>
            <Tooltip content={"Show snapshots"}>
                <Button isIconOnly size={"lg"} variant={showSnapshots ? "primary" : "secondary"} onPress={() => setShowSnapshots(prev => !prev)} isDisabled={props.isDisabled} className="rounded-none">
                    <Icon icon={"pixelarticons:bug"}/>
                </Button>
            </Tooltip>
            <Tooltip content={"Show Older Versions"}>
                <Button isIconOnly size={"lg"} variant={showOlderVersions ? "primary" : "secondary"} onPress={() => setShowOlderVersions(prev => !prev)} isDisabled={props.isDisabled} className="rounded-none">
                    <Icon icon={"pixelarticons:archive"}/>
                </Button>
            </Tooltip>
        </div>
    );
}