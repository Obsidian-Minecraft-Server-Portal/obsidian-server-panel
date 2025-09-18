import {Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useForgeVersions} from "../../../providers/LoaderVersionProviders/ForgeVersionsProvider.tsx";

type ForgeVersionSelectorProps = {
    minecraftVersion: string;
    version?: string;
    onVersionChange: (url: string | undefined, version: string | undefined) => void
    isDisabled: boolean
}

export function ForgeVersionSelector(props: ForgeVersionSelectorProps)
{
    const {minecraftVersion, version} = props;
    const {forgeVersions} = useForgeVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(version);
    const [versions, setVersions] = useState<string[]>([]);
    useEffect(() =>
    {
        if (!forgeVersions) return;
        const versions = forgeVersions[minecraftVersion].reverse(); // Reverse to show latest versions first
        if (versions && versions.length > 0)
        {
            setVersions(versions);
            // Only set default version if no version is controlled from parent
            if (!version && !selectedVersion)
            {
                setSelectedVersion(versions[0]);
            }
        } else
        {
            setVersions([]);
            setSelectedVersion(undefined);
        }
    }, [forgeVersions, minecraftVersion]); // Removed props from deps

    useEffect(() =>
    {
        if (!selectedVersion || !minecraftVersion) return;
        props.onVersionChange(getForgeInstallerUrl(selectedVersion), selectedVersion);
    }, [selectedVersion, minecraftVersion, props]);

    useEffect(() =>
    {
        if (version !== undefined)
        {
            setSelectedVersion(version);
        }
    }, [version]);

    return (
        <Autocomplete
            label={`Forge Version`}
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
            isDisabled={props.isDisabled}
            listboxProps={{
                emptyContent: `No Forge versions available for ${minecraftVersion}`,
                itemClasses: {
                    base: "rounded-none font-minecraft-body"
                }
            }}
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

function getForgeInstallerUrl(forgeVersion: string): string | undefined
{
    return `https://maven.minecraftforge.net/net/minecraftforge/forge/${forgeVersion}/forge-${forgeVersion}-installer.jar`;
}