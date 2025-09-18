import {Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useNeoForgeVersions} from "../../../providers/LoaderVersionProviders/NeoForgeVersionsProvider.tsx";

type NeoForgeVersionSelectorProps = {
    minecraftVersion: string;
    version?: string;
    onVersionChange?: (url: string | undefined, version: string | undefined) => void;
    isDisabled: boolean
}

export function NeoForgeVersionSelector(props: NeoForgeVersionSelectorProps)
{
    const {minecraftVersion, version, onVersionChange} = props;
    const {getFromMinecraftVersion} = useNeoForgeVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(version);
    const [versions, setVersions] = useState<string[]>([]);

    useEffect(() =>
    {
        const versions = getFromMinecraftVersion(minecraftVersion) as string[];
        if (versions && versions.length > 0)
        {
            setVersions(versions);
            // Only set default version if no version is controlled from parent
            if (!version && !selectedVersion) {
                setSelectedVersion(versions[0]);
            }
        } else
        {
            setVersions([]);
            setSelectedVersion(undefined);
        }
    }, [getFromMinecraftVersion, minecraftVersion]); // Removed props from deps

    useEffect(() =>
    {
        if (onVersionChange && selectedVersion && minecraftVersion)
        {
            // NeoForge installer URL structure
            const url = `https://maven.neoforged.net/releases/net/neoforged/neoforge/${selectedVersion}/neoforge-${selectedVersion}-installer.jar`;
            onVersionChange(url, selectedVersion);
        }
    }, [selectedVersion, onVersionChange, minecraftVersion]);

    useEffect(() =>
    {
        if (version !== undefined) {
            setSelectedVersion(version);
        }
    }, [version]);

    return (
        <Autocomplete
            label={`NeoForge Version`}
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
            listboxProps={{
                emptyContent: `No NeoForge versions available for ${minecraftVersion}`,
                itemClasses: {
                    base: "rounded-none font-minecraft-body"
                }
            }}

            isDisabled={props.isDisabled}
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
    );
}