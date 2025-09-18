import {addToast, Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useFabricVersions} from "../../../providers/LoaderVersionProviders/FabricVersionsProvider.tsx";
import {getFabricServerUrl} from "../../../ts/fabric-versions.ts";

type FabricVersionSelectorProps = {
    version?: string;
    minecraftVersion: string;
    isSnapshot: boolean;
    onVersionChange: (url: string | undefined, version: string | undefined) => void
    isDisabled: boolean
}

export function FabricVersionSelector(props: FabricVersionSelectorProps)
{
    const {minecraftVersion, version} = props;
    const {fabricVersions} = useFabricVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(version);
    const [versions, setVersions] = useState<string[]>([]);

    useEffect(() =>
    {
        if (!fabricVersions) return;
        if (+(minecraftVersion.split(".")[1]) < 14)
        {
            setVersions([]);
            setSelectedVersion(undefined);
            return;
        } // Fabric versions are not available for Minecraft versions below 1.14
        const versions = fabricVersions.loader.map(i => i.version);
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
    }, [fabricVersions, minecraftVersion]); // Removed props and selectedVersion from deps

    useEffect(() =>
    {
        let installer: string | undefined = fabricVersions?.installer?.find(i => i.stable)?.version;
        if (!installer)
        {
            addToast({
                title: "Error",
                description: "No stable Fabric installer version found.",
                color: "danger"
            });
            return;
        }
        if (!selectedVersion || !minecraftVersion) return;
        const url = getFabricServerUrl(selectedVersion, minecraftVersion, installer);
        props.onVersionChange(url, selectedVersion);
        console.log(`Selected Fabric version: ${selectedVersion}, URL: ${url}`);
    }, [selectedVersion, minecraftVersion, fabricVersions, props]);

    useEffect(() =>
    {
        if (version !== undefined) {
            setSelectedVersion(version);
        }
    }, [version]);

    return (
        <Autocomplete
            label={`Fabric Version`}
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
                emptyContent: `No Fabric versions available for Minecraft versions below 1.14`,
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