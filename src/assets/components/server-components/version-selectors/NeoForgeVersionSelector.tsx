import {Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useNeoForgeVersions} from "../../../providers/LoaderVersionProviders/NeoForgeVersionsProvider.tsx";

type ForgeVersionSelectorProps = {
    minecraftVersion: string;
    isDisabled:boolean
}

export function NeoForgeVersionSelector(props: ForgeVersionSelectorProps)
{
    const {minecraftVersion} = props;
    const {getFromMinecraftVersion} = useNeoForgeVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(undefined);
    const [versions, setVersions] = useState<string[]>([]);
    useEffect(() =>
    {
        const versions = getFromMinecraftVersion(minecraftVersion) as string[];
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
                emptyContent: `No NeoForge versions available for ${minecraftVersion}`
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