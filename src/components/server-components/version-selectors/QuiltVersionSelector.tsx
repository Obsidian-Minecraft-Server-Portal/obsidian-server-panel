import {addToast, Autocomplete, AutocompleteItem} from "@heroui/react";
import {useEffect, useState} from "react";
import {useQuiltVersions} from "../../../providers/LoaderVersionProviders/QuiltVersionsProvider.tsx";
import {getQuiltServerUrl} from "../../../ts/quilt-versions.ts";

type QuiltVersionSelectorProps = {
    minecraftVersion: string;
    version?: string;
    onVersionChange?: (url: string | undefined, version: string | undefined) => void;
    isDisabled: boolean
}

export function QuiltVersionSelector(props: QuiltVersionSelectorProps)
{
    const {minecraftVersion, version, onVersionChange} = props;
    const {quiltVersions} = useQuiltVersions();
    const [selectedVersion, setSelectedVersion] = useState<string | undefined>(version);
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
    }, [quiltVersions, minecraftVersion]); // Removed props from deps

    useEffect(() =>
    {
        let installer: string | undefined = quiltVersions?.installer?.find(i => i.stable)?.version;
        if (!installer)
        {
            addToast({
                title: "Error",
                description: "No stable Quilt installer version found.",
                color: "danger"
            });
            return;
        }
        if (!selectedVersion || !minecraftVersion) return;
        const url = getQuiltServerUrl(selectedVersion, minecraftVersion, installer);
        onVersionChange?.(url, selectedVersion);
        console.log(`Selected Quilt version: ${selectedVersion}, URL: ${url}`);
    }, [selectedVersion, minecraftVersion, quiltVersions, onVersionChange]);

    useEffect(() =>
    {
        if (version !== undefined)
        {
            setSelectedVersion(version);
        }
    }, [version]);

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
                emptyContent: `No Quilt versions available for Minecraft versions below 1.14`,
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