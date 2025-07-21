import {Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useForgeVersions} from "../../../providers/LoaderVersionProviders/ForgeVersionsProvider.tsx";

type ForgeVersionSelectorProps = {
    minecraftVersion: string;
}

export function ForgeVersionSelector(props: ForgeVersionSelectorProps)
{
    const {minecraftVersion} = props;
    const {forgeVersions} = useForgeVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(undefined);
    const [versions, setVersions] = useState<string[]>([]);
    useEffect(() =>
    {
        if (!forgeVersions) return;
        const versions = forgeVersions[minecraftVersion] as string[];
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
            listboxProps={{
                emptyContent: `No Forge versions available for ${minecraftVersion}`
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