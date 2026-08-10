import {useEffect, useState} from "react";
import {Icon} from "@iconify-icon/react";
import {getGetBukkitVersions, GetBukkitFlavor, GetBukkitVersion} from "../../../ts/getbukkit-versions.ts";

type GetBukkitVersionSelectorProps = {
    flavor: GetBukkitFlavor;
    minecraftVersion: string;
    onVersionChange?: (url: string | undefined, version: string | undefined) => void;
};

const FLAVOR_LABELS: Record<GetBukkitFlavor, string> = {
    spigot: "Spigot",
    craftbukkit: "CraftBukkit"
};

export function GetBukkitVersionSelector(props: GetBukkitVersionSelectorProps)
{
    const {flavor, minecraftVersion, onVersionChange} = props;
    const [versions, setVersions] = useState<GetBukkitVersion[] | null>(null);
    const [loadFailed, setLoadFailed] = useState(false);

    useEffect(() =>
    {
        let aborted = false;
        setVersions(null);
        setLoadFailed(false);
        getGetBukkitVersions(flavor)
            .then(list =>
            {
                if (!aborted) setVersions(list);
            })
            .catch(() =>
            {
                if (!aborted) setLoadFailed(true);
            });
        return () =>
        {
            aborted = true;
        };
    }, [flavor]);

    const match = versions?.find(v => v.version === minecraftVersion);

    useEffect(() =>
    {
        onVersionChange?.(match?.download_url, undefined);
    }, [match?.download_url, onVersionChange]);

    const label = FLAVOR_LABELS[flavor];

    if (loadFailed)
        return <p className={"text-danger font-minecraft-body text-tiny"}>Failed to load {label} versions from getbukkit.org. Please try again later.</p>;
    if (versions === null)
        return <p className={"text-default-500 font-minecraft-body text-tiny"}>Checking {label} availability for {minecraftVersion}...</p>;
    if (!match)
        return <p className={"text-danger font-minecraft-body text-tiny"}>{label} is not available for Minecraft {minecraftVersion}. Please select a different Minecraft version.</p>;

    return (
        <p className={"text-success font-minecraft-body text-tiny flex items-center gap-1"}>
            <Icon icon={"pixelarticons:check"}/> {label} {match.version} is available from getbukkit.org.
        </p>
    );
}
