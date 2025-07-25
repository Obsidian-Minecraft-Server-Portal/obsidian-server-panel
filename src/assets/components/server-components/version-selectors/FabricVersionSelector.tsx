import {addToast, Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useFabricVersions} from "../../../providers/LoaderVersionProviders/FabricVersionsProvider.tsx";
import {getFabricServerUrl} from "../../../ts/fabric-versions.ts";

type FabricVersionSelectorProps = {
    minecraftVersion: string;
    onVersionChange: (url: string | undefined, version: string | undefined) => void
    isDisabled: boolean
}

export function FabricVersionSelector(props: FabricVersionSelectorProps)
{
    const {minecraftVersion} = props;
    const {fabricVersions} = useFabricVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(undefined);
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
            setSelectedVersion(versions[0]);
        } else
        {
            setVersions([]);
            setSelectedVersion(undefined);
        }
    }, [props]);

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
        console.log(`Selected Fabric version: ${selectedVersion}, URL: ${url}`);
    }, [selectedVersion]);

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
                emptyContent: `No Fabric versions available for Minecraft versions below 1.14`
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