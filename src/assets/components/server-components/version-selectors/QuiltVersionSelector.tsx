import {Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useQuiltVersions} from "../../../providers/LoaderVersionProviders/QuiltVersionsProvider.tsx";

type QuiltVersionSelectorProps = {
    minecraftVersion: string;
    isDisabled: boolean
}

export function QuiltVersionSelector(props: QuiltVersionSelectorProps)
{
    const {minecraftVersion} = props;
    const {quiltVersions} = useQuiltVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(undefined);
    const [versions, setVersions] = useState<string[]>([]);
    useEffect(() =>
    {
        if (!quiltVersions) return;
        if (+(minecraftVersion.split(".")[1]) < 14)
        {
            setVersions([]);
            setSelectedVersion(undefined);
            return;
        } // Quilt versions are not available for Minecraft versions below 1.14
        const versions = quiltVersions.loader.map(i => i.version);
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
            label={`Quilt Version`}
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
                emptyContent: `No Quilt versions available for Minecraft versions below 1.14`
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